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

use std::borrow::Cow;
use std::sync::Arc;

use futures::StreamExt;
use nv_redfish::core::Bmc;
use nv_redfish::event_service::{Event, EventStreamPayload};

use crate::HealthError;
use crate::collectors::runtime::{ConnectFuture, EventStream, StreamingCollector, open_sse_stream};
use crate::endpoint::BmcEndpoint;
use crate::sink::{CollectorEvent, LogRecord};

pub struct SseLogCollectorConfig;

pub struct SseLogCollector<B: Bmc> {
    bmc: Arc<B>,
}

impl<B: Bmc + 'static> StreamingCollector<B> for SseLogCollector<B> {
    type Config = SseLogCollectorConfig;

    fn new_runner(
        bmc: Arc<B>,
        _endpoint: Arc<BmcEndpoint>,
        _config: Self::Config,
    ) -> Result<Self, HealthError> {
        Ok(Self { bmc })
    }

    fn connect(&mut self) -> ConnectFuture<'_> {
        Box::pin(async move {
            let sse_stream = open_sse_stream(Arc::clone(&self.bmc)).await?;

            let bmc = Arc::clone(&self.bmc);
            let event_stream: EventStream<'_> = sse_stream
                .flat_map(move |result| {
                    let events = map_payload(result, bmc.as_ref());
                    futures::stream::iter(events)
                })
                .boxed();

            Ok(event_stream)
        })
    }

    fn collector_type(&self) -> &'static str {
        "sse_logs"
    }
}

fn map_payload<B: Bmc>(
    result: Result<EventStreamPayload, HealthError>,
    bmc: &B,
) -> Vec<Result<CollectorEvent, HealthError>> {
    match result {
        Ok(EventStreamPayload::Event(event)) => event_to_logs(&event, bmc),
        Ok(EventStreamPayload::MetricReport(_)) => Vec::new(),
        Err(e) => vec![Err(e)],
    }
}

fn event_to_logs<B: Bmc>(event: &Event, bmc: &B) -> Vec<Result<CollectorEvent, HealthError>> {
    event
        .events
        .iter()
        .flat_map(|nav| futures::FutureExt::now_or_never(nav.get(bmc)))
        .filter_map(|result| result.ok())
        .map(|record| {
            let body = record.message.as_deref().unwrap_or("").to_string();

            let severity = record.severity.as_deref().unwrap_or("Unknown").to_string();

            let mut attributes = vec![
                (Cow::Borrowed("message_id"), record.message_id.clone()),
                (
                    Cow::Borrowed("event_type"),
                    format!("{:?}", record.event_type),
                ),
            ];
            if let Some(event_id) = &record.event_id {
                attributes.push((Cow::Borrowed("event_id"), event_id.clone()));
            }
            if let Some(timestamp) = &record.event_timestamp {
                attributes.push((Cow::Borrowed("event_timestamp"), timestamp.to_string()));
            }

            Ok(CollectorEvent::Log(Box::new(LogRecord {
                body,
                severity,
                attributes,
            })))
        })
        .collect()
}
