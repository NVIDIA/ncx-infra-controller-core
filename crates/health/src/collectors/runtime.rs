/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use http::header::InvalidHeaderValue;
use http::{HeaderMap, header};
use nv_redfish::ServiceRoot;
use nv_redfish::bmc_http::reqwest::Client as ReqwestClient;
use nv_redfish::bmc_http::{CacheSettings, HttpBmc};
use nv_redfish::core::Bmc;
use nv_redfish::event_service::EventStreamPayload;
use prometheus::{Counter, Gauge, Histogram, HistogramOpts, Opts};
use rand::Rng;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use crate::HealthError;
use crate::config::Config as AppConfig;
use crate::discovery::BmcClient;
use crate::endpoint::BmcEndpoint;
use crate::limiter::RateLimiter;
use crate::metrics::{CollectorRegistry, operation_duration_buckets_seconds};
use crate::sink::{CollectorEvent, DataSink, EventContext};

/// Result of a collector iteration
#[derive(Debug, Clone)]
pub struct IterationResult {
    /// Whether a refresh was triggered (data was fetched vs cached)
    pub refresh_triggered: bool,
    /// Number of entities collected, if applicable
    pub entity_count: Option<usize>,
}

pub trait PeriodicCollector<B: Bmc>: Send + 'static {
    type Config: Send + 'static;

    fn new_runner(
        bmc: Arc<B>,
        endpoint: Arc<BmcEndpoint>,
        config: Self::Config,
    ) -> Result<Self, HealthError>
    where
        Self: Sized;

    fn run_iteration(
        &mut self,
    ) -> impl std::future::Future<Output = Result<IterationResult, HealthError>> + Send;

    /// Returns the type identifier for this collector
    fn collector_type(&self) -> &'static str;
}

pub type EventStream<'a> = BoxStream<'a, Result<CollectorEvent, HealthError>>;

pub type ConnectFuture<'a> =
    Pin<Box<dyn std::future::Future<Output = Result<EventStream<'a>, HealthError>> + Send + 'a>>;

/// Trait for collectors that maintain a long-lived stream (SSE, gRPC, etc.)
/// runtime.rs creates the BMC client and injects it, the collector opens the stream and maps payloads to events
pub trait StreamingCollector<B: Bmc>: Send + 'static {
    type Config: Send + 'static;

    fn new_runner(
        bmc: Arc<B>,
        endpoint: Arc<BmcEndpoint>,
        config: Self::Config,
    ) -> Result<Self, HealthError>
    where
        Self: Sized;

    /// Open or reopen the streaming connection using the injected BMC.
    fn connect(&mut self) -> ConnectFuture<'_>;

    fn collector_type(&self) -> &'static str;
}

pub struct BackoffConfig {
    pub initial: Duration,
    pub max: Duration,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(30),
        }
    }
}

pub struct ExponentialBackoff {
    initial: Duration,
    max: Duration,
    current: Duration,
}

impl ExponentialBackoff {
    pub fn new(config: &BackoffConfig) -> Self {
        Self {
            initial: config.initial,
            max: config.max,
            current: config.initial,
        }
    }

    pub fn next_delay(&mut self) -> Duration {
        let base = self.current;
        self.current = (self.current * 2).min(self.max);
        let jitter_ms = rand::rng().random_range(0..base.as_millis().max(1) as u64);
        base + Duration::from_millis(jitter_ms)
    }

    pub fn reset(&mut self) {
        self.current = self.initial;
    }
}

pub type SseStream = Pin<
    Box<
        dyn futures::TryStream<
                Ok = EventStreamPayload,
                Error = HealthError,
                Item = Result<EventStreamPayload, HealthError>,
            > + Send,
    >,
>;

/// Open a Redfish SSE event stream from a BMC.
pub async fn open_sse_stream<B: Bmc + 'static>(bmc: Arc<B>) -> Result<SseStream, HealthError> {
    let root = ServiceRoot::new(bmc)
        .await
        .map_err(|e| HealthError::BmcError(Box::new(e)))?;

    let event_service = root
        .event_service()
        .await
        .map_err(|e| HealthError::BmcError(Box::new(e)))?
        .ok_or_else(|| {
            HealthError::SseNotAvailable("BMC does not expose an EventService".to_string())
        })?;

    let stream = event_service
        .events()
        .await
        .map_err(|e| HealthError::BmcError(Box::new(e)))?;

    Ok(Box::pin(
        stream.map_err(|e| HealthError::BmcError(Box::new(e))),
    ))
}

// SSE EventSource readyState values for the connection_state gauge
// ref: https://html.spec.whatwg.org/multipage/server-sent-events.html#dom-eventsource-readystate
const STREAM_STATE_CONNECTING: f64 = 0.0;
const STREAM_STATE_OPEN: f64 = 1.0;
const STREAM_STATE_CLOSED: f64 = 2.0;

pub struct StreamMetrics {
    connection_state: Gauge,
    reconnections_total: Counter,
    items_processed_total: Counter,
    stream_errors_total: Counter,
}

impl StreamMetrics {
    fn new(
        registry: &prometheus::Registry,
        prefix: &str,
        const_labels: HashMap<String, String>,
    ) -> Result<Self, HealthError> {
        let connection_state = Gauge::with_opts(
            Opts::new(
                format!("{prefix}_stream_connection_state"),
                "Stream connection state per SSE readyState: 0=CONNECTING, 1=OPEN, 2=CLOSED",
            )
            .const_labels(const_labels.clone()),
        )?;
        registry.register(Box::new(connection_state.clone()))?;

        let reconnections_total = Counter::with_opts(
            Opts::new(
                format!("{prefix}_stream_reconnections_total"),
                "Total reconnection attempts",
            )
            .const_labels(const_labels.clone()),
        )?;
        registry.register(Box::new(reconnections_total.clone()))?;

        let items_processed_total = Counter::with_opts(
            Opts::new(
                format!("{prefix}_stream_items_processed_total"),
                "Total stream items processed",
            )
            .const_labels(const_labels.clone()),
        )?;
        registry.register(Box::new(items_processed_total.clone()))?;

        let stream_errors_total = Counter::with_opts(
            Opts::new(
                format!("{prefix}_stream_errors_total"),
                "Total stream errors",
            )
            .const_labels(const_labels),
        )?;
        registry.register(Box::new(stream_errors_total.clone()))?;

        Ok(Self {
            connection_state,
            reconnections_total,
            items_processed_total,
            stream_errors_total,
        })
    }
}

pub struct Collector {
    handle: JoinHandle<()>,
    cancel_token: CancellationToken,
}

impl Collector {
    pub fn start<C: PeriodicCollector<BmcClient>>(
        endpoint: Arc<BmcEndpoint>,
        limiter: Arc<dyn RateLimiter>,
        iteration_interval: Duration,
        config: C::Config,
        collector_registry: Arc<CollectorRegistry>,
        client: ReqwestClient,
        health_options: &AppConfig,
    ) -> Result<Self, HealthError> {
        let cancel_token = CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();

        let bmc_url = match &health_options.bmc_proxy_url {
            Some(url) => url.clone(),
            None => endpoint
                .addr
                .to_url()
                .map_err(|e| HealthError::GenericError(e.to_string()))?,
        };

        let headers = if health_options.bmc_proxy_url.is_some() {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::FORWARDED,
                format!("host={}", endpoint.addr.ip)
                    .parse()
                    .map_err(|e: InvalidHeaderValue| HealthError::GenericError(e.to_string()))?,
            );
            headers
        } else {
            HeaderMap::new()
        };

        let bmc = Arc::new(HttpBmc::with_custom_headers(
            client,
            bmc_url,
            endpoint.credentials.clone().into(),
            CacheSettings::with_capacity(health_options.cache_size),
            headers,
        ));

        let mut runner = C::new_runner(bmc, endpoint.clone(), config)?;

        let endpoint_key = endpoint.addr.hash_key().to_string();
        let const_labels = HashMap::from([
            (
                "collector_type".to_string(),
                runner.collector_type().to_string(),
            ),
            ("endpoint_key".to_string(), endpoint_key),
        ]);

        let registry = collector_registry.registry();

        let iteration_histogram = Histogram::with_opts(
            HistogramOpts::new(
                format!(
                    "{}_collector_iteration_seconds",
                    collector_registry.prefix()
                ),
                "Duration of collector iterations",
            )
            .const_labels(const_labels.clone())
            .buckets(operation_duration_buckets_seconds()),
        )?;
        registry.register(Box::new(iteration_histogram.clone()))?;

        let refresh_counter = Counter::with_opts(
            Opts::new(
                format!("{}_collector_refresh_total", collector_registry.prefix()),
                "Count of collector refreshes",
            )
            .const_labels(const_labels.clone()),
        )?;
        registry.register(Box::new(refresh_counter.clone()))?;

        let entities_gauge = Gauge::with_opts(
            Opts::new(
                format!("{}_monitored_entities", collector_registry.prefix()),
                "Number of entities being monitored",
            )
            .const_labels(const_labels),
        )?;
        registry.register(Box::new(entities_gauge.clone()))?;

        let handle = tokio::spawn(async move {
            let collector_type = runner.collector_type();
            // We keep to prevent registry clean_up on drop, after handle is dropped,
            // collector_registry will be dropped and unregister the metrics.
            let _collector_registry = collector_registry;
            loop {
                tokio::select! {
                    _ = cancel_token_clone.cancelled() => {
                        tracing::info!("Collector cancelled for addr: {:?}", endpoint.addr);
                        break;
                    }
                    _ = async {
                        limiter.acquire().await;

                        let start = Instant::now();
                        let iteration_result = runner.run_iteration().await;
                        let duration = start.elapsed();

                        iteration_histogram.observe(
                            duration.as_secs_f64(),
                        );

                        match iteration_result {
                            Ok(result) => {
                                if result.refresh_triggered {
                                    refresh_counter.inc();
                                }

                                if let Some(entity_count) = result.entity_count {
                                    entities_gauge.set(
                                        entity_count as f64,
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    error = ?e,
                                    endpoint = ?endpoint.addr,
                                    collector_type = collector_type,
                                    "Error during collector iteration"
                                );
                            }
                        }

                        tokio::time::sleep(iteration_interval).await;
                    } => {
                        // Iteration completed
                    }
                }
            }
        });

        Ok(Self {
            handle,
            cancel_token,
        })
    }

    pub fn start_streaming<S: StreamingCollector<BmcClient>>(
        endpoint: Arc<BmcEndpoint>,
        config: S::Config,
        data_sink: Arc<dyn DataSink>,
        backoff_config: BackoffConfig,
        collector_registry: Arc<CollectorRegistry>,
        client: ReqwestClient,
        health_options: &AppConfig,
    ) -> Result<Self, HealthError> {
        let cancel_token = CancellationToken::new();
        let cancel_clone = cancel_token.clone();

        let bmc_url = match &health_options.bmc_proxy_url {
            Some(url) => url.clone(),
            None => endpoint
                .addr
                .to_url()
                .map_err(|e| HealthError::GenericError(e.to_string()))?,
        };

        let headers = if health_options.bmc_proxy_url.is_some() {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::FORWARDED,
                format!("host={}", endpoint.addr.ip)
                    .parse()
                    .map_err(|e: InvalidHeaderValue| HealthError::GenericError(e.to_string()))?,
            );
            headers
        } else {
            HeaderMap::new()
        };

        let bmc = Arc::new(HttpBmc::with_custom_headers(
            client,
            bmc_url,
            endpoint.credentials.clone().into(),
            CacheSettings::with_capacity(health_options.cache_size),
            headers,
        ));

        let mut collector = S::new_runner(bmc, endpoint.clone(), config)?;
        let event_context = EventContext::from_endpoint(&endpoint, collector.collector_type());

        let endpoint_key = endpoint.addr.hash_key().to_string();
        let const_labels = HashMap::from([
            (
                "collector_type".to_string(),
                collector.collector_type().to_string(),
            ),
            ("endpoint_key".to_string(), endpoint_key),
        ]);

        let registry = collector_registry.registry();
        let metrics = StreamMetrics::new(registry, collector_registry.prefix(), const_labels)?;

        let handle = tokio::spawn(async move {
            let collector_type = collector.collector_type();
            let _collector_registry = collector_registry;
            let mut backoff = ExponentialBackoff::new(&backoff_config);

            loop {
                metrics.connection_state.set(STREAM_STATE_CONNECTING);
                tracing::info!(
                    collector_type,
                    endpoint = ?endpoint.addr,
                    "streaming collector connecting"
                );

                let stream = tokio::select! {
                    _ = cancel_clone.cancelled() => {
                        metrics.connection_state.set(STREAM_STATE_CLOSED);
                        return;
                    }
                    result = collector.connect() => result,
                };

                let mut stream = match stream {
                    Ok(s) => {
                        metrics.connection_state.set(STREAM_STATE_OPEN);
                        backoff.reset();
                        tracing::info!(
                            collector_type,
                            endpoint = ?endpoint.addr,
                            "streaming collector connected"
                        );
                        s
                    }
                    Err(e) => {
                        metrics.reconnections_total.inc();
                        tracing::error!(
                            error = ?e,
                            collector_type,
                            endpoint = ?endpoint.addr,
                            "streaming collector connection failed"
                        );
                        let delay = backoff.next_delay();
                        tokio::select! {
                            _ = cancel_clone.cancelled() => {
                                metrics.connection_state.set(STREAM_STATE_CLOSED);
                                return;
                            }
                            _ = tokio::time::sleep(delay) => continue,
                        }
                    }
                };

                loop {
                    let item = tokio::select! {
                        _ = cancel_clone.cancelled() => {
                            metrics.connection_state.set(STREAM_STATE_CLOSED);
                            tracing::info!(
                                collector_type,
                                endpoint = ?endpoint.addr,
                                "streaming collector shutting down"
                            );
                            return;
                        }
                        item = stream.next() => item,
                    };

                    match item {
                        Some(Ok(event)) => {
                            metrics.items_processed_total.inc();
                            data_sink.handle_event(&event_context, &event);
                        }
                        Some(Err(e)) => {
                            metrics.stream_errors_total.inc();
                            metrics.reconnections_total.inc();
                            tracing::error!(
                                error = ?e,
                                collector_type,
                                endpoint = ?endpoint.addr,
                                "streaming collector stream error, reconnecting"
                            );
                            break;
                        }
                        None => {
                            tracing::info!(
                                collector_type,
                                endpoint = ?endpoint.addr,
                                "streaming collector stream ended, reconnecting"
                            );
                            break;
                        }
                    }
                }

                // stream ended or errored -- transition to CONNECTING and retry
                metrics.connection_state.set(STREAM_STATE_CONNECTING);
                let delay = backoff.next_delay();
                tokio::select! {
                    _ = cancel_clone.cancelled() => {
                        metrics.connection_state.set(STREAM_STATE_CLOSED);
                        return;
                    }
                    _ = tokio::time::sleep(delay) => {}
                }
            }
        });

        Ok(Self {
            handle,
            cancel_token,
        })
    }

    pub async fn stop(self) {
        self.cancel_token.cancel();
        let _ = self.handle.await;
    }
}
