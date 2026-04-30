/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;
use std::time::Instant;

use super::{CollectorEvent, DataSink, EventContext};
use crate::metrics::{ComponentKind, ComponentMetrics, MetricsManager};

pub struct CompositeDataSink {
    sinks: Vec<Arc<dyn DataSink>>,
    component_metrics: Arc<ComponentMetrics>,
}

impl CompositeDataSink {
    pub fn new(sinks: Vec<Arc<dyn DataSink>>, metrics_manager: Arc<MetricsManager>) -> Self {
        Self {
            sinks,
            component_metrics: metrics_manager.component_metrics(),
        }
    }

    fn record_sink_operation(&self, sink: &dyn DataSink, duration: std::time::Duration) {
        self.component_metrics.record_operation(
            ComponentKind::Sink,
            sink.sink_type(),
            duration,
            true,
        );
    }
}

impl DataSink for CompositeDataSink {
    fn sink_type(&self) -> &'static str {
        "composite_sink"
    }

    fn handle_event(&self, context: &EventContext, event: &CollectorEvent) {
        for sink in &self.sinks {
            let start = Instant::now();
            sink.handle_event(context, event);
            self.record_sink_operation(sink.as_ref(), start.elapsed());
        }
    }
}
