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

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::processor::EventProcessingPipeline;
use crate::sink::{CollectorEvent, DataSink, EventContext};

/// sync pipeline + optional bounded OTLP channel. the channel send is the
/// backpressure point; sync sinks always complete before it.
pub struct EventPipeline {
    inner: Arc<EventProcessingPipeline>,
    otlp_sender: Option<mpsc::Sender<(EventContext, CollectorEvent)>>,
}

impl EventPipeline {
    pub fn new(
        inner: EventProcessingPipeline,
        otlp_sender: Option<mpsc::Sender<(EventContext, CollectorEvent)>>,
    ) -> Self {
        Self {
            inner: Arc::new(inner),
            otlp_sender,
        }
    }

    pub async fn handle_event(&self, context: &EventContext, event: &CollectorEvent) {
        let Some(sender) = &self.otlp_sender else {
            self.inner.handle_event(context, event);
            return;
        };

        // TODO: fix waste: context is cloned per otlp-relevant event
        // probably better to switch channel to Arc<EventContext> to pay for one allocation instead of N clones
        for evt in self.inner.handle_and_collect(context, event) {
            if is_otlp_relevant(&evt) && sender.send((context.clone(), evt)).await.is_err() {
                tracing::warn!("otlp channel closed, event dropped");
                break;
            }
        }
    }
}

fn is_otlp_relevant(event: &CollectorEvent) -> bool {
    !matches!(
        event,
        CollectorEvent::Metric(_)
            | CollectorEvent::MetricCollectionStart
            | CollectorEvent::MetricCollectionEnd
    )
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::str::FromStr;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use mac_address::MacAddress;

    use super::*;
    use crate::endpoint::BmcAddr;
    use crate::metrics::MetricsManager;
    use crate::sink::{DataSink, LogRecord};

    fn test_metrics_manager() -> Arc<MetricsManager> {
        Arc::new(MetricsManager::new("test").expect("should create metrics manager"))
    }

    struct CountingSink {
        counter: Arc<AtomicUsize>,
    }

    impl DataSink for CountingSink {
        fn handle_event(&self, _: &EventContext, _: &CollectorEvent) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn test_context() -> EventContext {
        EventContext {
            endpoint_key: "42:9e:b1:bd:9d:dd".to_string(),
            addr: BmcAddr {
                ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
                port: Some(443),
                mac: MacAddress::from_str("42:9e:b1:bd:9d:dd").expect("valid mac"),
            },
            collector_type: "test",
            metadata: None,
        }
    }

    fn log_event() -> CollectorEvent {
        CollectorEvent::Log(Box::new(LogRecord {
            body: "test".to_string(),
            severity: "INFO".to_string(),
            attributes: vec![],
        }))
    }

    #[tokio::test]
    async fn sync_sinks_complete_before_otlp_send() {
        let sink_counter = Arc::new(AtomicUsize::new(0));
        let sink: Arc<dyn DataSink> = Arc::new(CountingSink {
            counter: sink_counter.clone(),
        });
        let inner = EventProcessingPipeline::new(vec![], sink, test_metrics_manager());
        let (otlp_tx, mut otlp_rx) = tokio::sync::mpsc::channel(10);
        let pipeline = EventPipeline::new(inner, Some(otlp_tx));

        pipeline.handle_event(&test_context(), &log_event()).await;

        assert_eq!(sink_counter.load(Ordering::SeqCst), 1);
        assert!(otlp_rx.try_recv().is_ok());
    }

    #[tokio::test]
    async fn metric_events_are_filtered_from_otlp_channel() {
        let sink: Arc<dyn DataSink> = Arc::new(CountingSink {
            counter: Arc::new(AtomicUsize::new(0)),
        });
        let inner = EventProcessingPipeline::new(vec![], sink, test_metrics_manager());
        let (otlp_tx, mut otlp_rx) = tokio::sync::mpsc::channel(10);
        let pipeline = EventPipeline::new(inner, Some(otlp_tx));

        pipeline
            .handle_event(&test_context(), &CollectorEvent::MetricCollectionStart)
            .await;

        assert!(otlp_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn works_without_otlp_sender() {
        let sink_counter = Arc::new(AtomicUsize::new(0));
        let sink: Arc<dyn DataSink> = Arc::new(CountingSink {
            counter: sink_counter.clone(),
        });
        let inner = EventProcessingPipeline::new(vec![], sink, test_metrics_manager());
        let pipeline = EventPipeline::new(inner, None);

        pipeline.handle_event(&test_context(), &log_event()).await;

        assert_eq!(sink_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn backpressure_suspends_on_full_channel() {
        let sink: Arc<dyn DataSink> = Arc::new(CountingSink {
            counter: Arc::new(AtomicUsize::new(0)),
        });
        let inner = EventProcessingPipeline::new(vec![], sink, test_metrics_manager());
        let (otlp_tx, _otlp_rx) = tokio::sync::mpsc::channel(1);
        let pipeline = Arc::new(EventPipeline::new(inner, Some(otlp_tx)));

        let event = log_event();
        pipeline.handle_event(&test_context(), &event).await;

        let pipeline_clone = Arc::clone(&pipeline);
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(50),
            pipeline_clone.handle_event(&test_context(), &event),
        )
        .await;

        assert!(result.is_err(), "expected timeout due to full channel");
    }

    #[tokio::test]
    async fn processor_derived_events_reach_otlp_channel() {
        use crate::processor::EventProcessor;

        struct DoubleProcessor;
        impl EventProcessor for DoubleProcessor {
            fn process_event(
                &self,
                _: &EventContext,
                event: &CollectorEvent,
            ) -> Vec<CollectorEvent> {
                vec![event.clone()]
            }
        }

        let sink_counter = Arc::new(AtomicUsize::new(0));
        let sink: Arc<dyn DataSink> = Arc::new(CountingSink {
            counter: sink_counter.clone(),
        });
        let inner = EventProcessingPipeline::new(
            vec![Arc::new(DoubleProcessor)],
            sink,
            test_metrics_manager(),
        );
        let (otlp_tx, mut otlp_rx) = tokio::sync::mpsc::channel(10);
        let pipeline = EventPipeline::new(inner, Some(otlp_tx));

        pipeline.handle_event(&test_context(), &log_event()).await;

        assert_eq!(sink_counter.load(Ordering::SeqCst), 2);
        assert!(otlp_rx.try_recv().is_ok());
        assert!(otlp_rx.try_recv().is_ok());
        assert!(otlp_rx.try_recv().is_err());
    }
}
