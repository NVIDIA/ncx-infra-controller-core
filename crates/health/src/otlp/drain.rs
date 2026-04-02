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

use std::time::Duration;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tonic::transport::Channel;

use super::collector_logs::logs_service_client::LogsServiceClient;
use super::convert::build_export_request;
use crate::collectors::{BackoffConfig, ExponentialBackoff};
use crate::sink::{CollectorEvent, EventContext};

pub struct OtlpDrainTask {
    receiver: mpsc::Receiver<(EventContext, CollectorEvent)>,
    cancel: CancellationToken,
    endpoint: String,
    batch_size: usize,
    flush_interval: Duration,
}

impl OtlpDrainTask {
    pub fn new(
        receiver: mpsc::Receiver<(EventContext, CollectorEvent)>,
        cancel: CancellationToken,
        endpoint: String,
        batch_size: usize,
        flush_interval: Duration,
    ) -> Self {
        Self {
            receiver,
            cancel,
            endpoint,
            batch_size,
            flush_interval,
        }
    }

    pub async fn run(mut self) {
        let mut client = match self.connect().await {
            Some(c) => c,
            None => return,
        };

        let mut batch = Vec::with_capacity(self.batch_size);
        let sleep = tokio::time::sleep(self.flush_interval);
        tokio::pin!(sleep);

        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => {
                    self.flush(&mut client, &mut batch).await;
                    break;
                }
                maybe_event = self.receiver.recv() => {
                    match maybe_event {
                        Some(event) => {
                            batch.push(event);
                            if batch.len() >= self.batch_size {
                                self.flush(&mut client, &mut batch).await;
                                sleep.as_mut().reset(tokio::time::Instant::now() + self.flush_interval);
                            }
                        }
                        None => {
                            self.flush(&mut client, &mut batch).await;
                            break;
                        }
                    }
                }
                _ = &mut sleep => {
                    if !batch.is_empty() {
                        self.flush(&mut client, &mut batch).await;
                    }
                    sleep.as_mut().reset(tokio::time::Instant::now() + self.flush_interval);
                }
            }
        }
    }

    async fn connect(&self) -> Option<LogsServiceClient<Channel>> {
        let endpoint = match Channel::from_shared(self.endpoint.clone()) {
            Ok(e) => e,
            Err(error) => {
                tracing::error!(
                    ?error,
                    endpoint = %self.endpoint,
                    "invalid otlp endpoint uri, stopping drain"
                );
                return None;
            }
        };

        let mut backoff = ExponentialBackoff::new(&BackoffConfig {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(30),
        });

        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => return None,
                result = endpoint.connect() => {
                    match result {
                        Ok(channel) => {
                            tracing::info!(endpoint = %self.endpoint, "connected to otlp collector");
                            return Some(LogsServiceClient::new(channel));
                        }
                        Err(error) => {
                            let delay = backoff.next_delay();
                            tracing::warn!(
                                ?error,
                                endpoint = %self.endpoint,
                                retry_in = ?delay,
                                "failed to connect to otlp collector"
                            );
                            tokio::select! {
                                _ = self.cancel.cancelled() => return None,
                                _ = tokio::time::sleep(delay) => {}
                            }
                        }
                    }
                }
            }
        }
    }

    async fn flush(
        &self,
        client: &mut LogsServiceClient<Channel>,
        batch: &mut Vec<(EventContext, CollectorEvent)>,
    ) {
        if batch.is_empty() {
            return;
        }

        let request = build_export_request(batch);
        let record_count = request
            .resource_logs
            .iter()
            .flat_map(|rl| &rl.scope_logs)
            .map(|sl| sl.log_records.len())
            .sum::<usize>();

        if record_count == 0 {
            batch.clear();
            return;
        }

        const MAX_RETRIES: usize = 5;

        let mut backoff = ExponentialBackoff::new(&BackoffConfig {
            initial: Duration::from_millis(100),
            max: Duration::from_secs(10),
        });

        for attempt in 0..=MAX_RETRIES {
            match client.export(request.clone()).await {
                Ok(_) => {
                    tracing::debug!(record_count, "exported logs to otlp collector");
                    break;
                }
                Err(status) if is_retryable(&status) && attempt < MAX_RETRIES => {
                    let delay = backoff.next_delay();
                    tracing::warn!(
                        code = ?status.code(),
                        message = status.message(),
                        attempt,
                        retry_in = ?delay,
                        "retryable otlp export error"
                    );
                    tokio::select! {
                        _ = self.cancel.cancelled() => break,
                        _ = tokio::time::sleep(delay) => continue,
                    }
                }
                Err(status) => {
                    tracing::error!(
                        code = ?status.code(),
                        message = status.message(),
                        record_count,
                        attempt,
                        "otlp export failed, dropping batch"
                    );
                    break;
                }
            }
        }

        batch.clear();
    }
}

fn is_retryable(status: &tonic::Status) -> bool {
    matches!(
        status.code(),
        tonic::Code::Unavailable
            | tonic::Code::DeadlineExceeded
            | tonic::Code::ResourceExhausted
            | tonic::Code::Aborted
            | tonic::Code::Internal
    )
}
