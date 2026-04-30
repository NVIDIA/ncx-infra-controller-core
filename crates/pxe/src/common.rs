/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use axum_template::engine::Engine;
use carbide_uuid::machine::MachineInterfaceId;
use metrics_exporter_prometheus::PrometheusHandle;
use rpc::forge::CloudInitInstructions;
use serde::{Deserialize, Serialize};
use tera::Tera;

use crate::config::RuntimeConfig;
use crate::extractors::machine_architecture;
// use crate::middleware::metrics::RequestMetrics;

#[derive(Debug)]
pub(crate) struct Machine {
    pub instructions: CloudInitInstructions,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MachineInterface {
    pub architecture: Option<machine_architecture::MachineArchitecture>,
    pub interface_id: MachineInterfaceId,
    pub platform: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial: Option<String>,
    pub asset: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub engine: Engine<Tera>,
    // pub request_metrics: RequestMetrics,
    pub runtime_config: RuntimeConfig,
    pub prometheus_handle: PrometheusHandle,
}
