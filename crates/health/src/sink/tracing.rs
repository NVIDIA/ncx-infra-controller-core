/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::{CollectorEvent, DataSink, EventContext};

pub struct TracingSink;

impl DataSink for TracingSink {
    fn sink_type(&self) -> &'static str {
        "tracing_sink"
    }

    fn handle_event(&self, context: &EventContext, event: &CollectorEvent) {
        match event {
            CollectorEvent::MetricCollectionStart => {
                tracing::info!(
                    endpoint = %context.endpoint_key(),
                    collector = %context.collector_type,
                    "Metric collection start"
                );
            }
            CollectorEvent::Metric(metric) => {
                tracing::info!(
                    endpoint = %context.endpoint_key(),
                    collector = %context.collector_type,
                    metric = %metric.name,
                    key = %metric.key,
                    metric_type = %metric.metric_type,
                    unit = %metric.unit,
                    value = metric.value,
                    "Metric event"
                );
            }
            CollectorEvent::MetricCollectionEnd => {
                tracing::info!(
                    endpoint = %context.endpoint_key(),
                    collector = %context.collector_type,
                    "Metric collection end"
                );
            }
            CollectorEvent::Log(record) => {
                tracing::info!(
                    endpoint = %context.endpoint_key(),
                    collector = %context.collector_type,
                    severity = %record.severity,
                    body = %record.body,
                    "Log event"
                );
            }
            CollectorEvent::Firmware(info) => {
                tracing::info!(
                    endpoint = %context.endpoint_key(),
                    collector = %context.collector_type,
                    component = %info.component,
                    version = %info.version,
                    "Firmware info event"
                );
            }
            CollectorEvent::HealthReport(report) => {
                tracing::info!(
                    endpoint = %context.endpoint_key(),
                    collector = %context.collector_type,
                    machine_id = ?context.machine_id(),
                    success_count = report.successes.len(),
                    alert_count = report.alerts.len(),
                    alerts = ?report.alerts,
                    report_source = report.source.as_str(),
                    "Health report event"
                );
            }
        }
    }
}
