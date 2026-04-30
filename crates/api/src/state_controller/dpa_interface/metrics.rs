/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Defines custom metrics that are collected and emitted by the Machine State Controller

use ::carbide_utils::metrics::SharedMetricsHolder;
use opentelemetry::metrics::Meter;

use crate::state_controller::metrics::MetricsEmitter;

#[derive(Debug, Default, Clone)]
pub struct DpaInterfaceMetrics {}

#[derive(Debug, Default)]
pub struct DpaInterfaceStateControllerIterationMetrics {}

#[derive(Debug)]
pub struct DpaInterfaceMetricsEmitter {}

impl DpaInterfaceStateControllerIterationMetrics {}

impl MetricsEmitter for DpaInterfaceMetricsEmitter {
    type ObjectMetrics = DpaInterfaceMetrics;
    type IterationMetrics = DpaInterfaceStateControllerIterationMetrics;

    fn new(
        _object_type: &str,
        _meter: &Meter,
        _shared_metrics: SharedMetricsHolder<Self::IterationMetrics>,
    ) -> Self {
        Self {}
    }

    // This routine is called in the context of a single thread.
    // The statecontroller launches multiple threads (upto max_concurrency)
    // Each thread works on one object and records the metrics for that object.
    // Once all the tasks are done, the original thread calls merge object_handling_metrics.
    // No need for mutex when manipulating the seg_stats HashMap.
    fn merge_object_handling_metrics(
        _iteration_metrics: &mut Self::IterationMetrics,
        _object_metrics: &Self::ObjectMetrics,
    ) {
    }

    fn emit_object_counters_and_histograms(&self, _object_metrics: &Self::ObjectMetrics) {}
}
