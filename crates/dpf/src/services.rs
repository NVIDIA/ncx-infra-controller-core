/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! DPU service definitions (DTS, etc.) for DPUServiceTemplate and DPUServiceConfiguration.

use crate::types::{
    ConfigPortsServiceType, ServiceConfigPort, ServiceConfigPortProtocol, ServiceDefinition,
};

/// Default DOCA helm registry (DPUServiceTemplate source.repoURL).
pub const DEFAULT_DOCA_HELM_REGISTRY: &str = "https://helm.ngc.nvidia.com/nvidia/doca";

/// Overridable registry configuration for DPU services.
///
/// Allows callers to redirect helm chart sources for airgapped,
/// development, or mirrored environments.
#[derive(Debug, Clone)]
pub struct ServiceRegistryConfig {
    /// Helm chart repository URL for DOCA services (HBN, DTS).
    pub doca_helm_registry: String,
}

impl Default for ServiceRegistryConfig {
    fn default() -> Self {
        Self {
            doca_helm_registry: DEFAULT_DOCA_HELM_REGISTRY.to_string(),
        }
    }
}

/// DTS (Doca Telemetry Service) service definition.
pub fn dts_service(reg: &ServiceRegistryConfig) -> ServiceDefinition {
    ServiceDefinition {
        helm_values: Some(serde_json::json!({
            "exposedPorts": { "ports": { "httpserverport": true } },
            "serviceDaemonSet": {
                "resources": {
                    "requests": { "memory": "320Mi", "cpu": "200m" },
                    "limits":   { "memory": "320Mi", "cpu": "1" }
                }
            }

        })),
        config_ports: Some(vec![ServiceConfigPort {
            name: "httpserverport".to_string(),
            port: 9100,
            protocol: ServiceConfigPortProtocol::Tcp,
            node_port: None,
        }]),
        config_ports_service_type: Some(ConfigPortsServiceType::None),
        ..ServiceDefinition::new("dts", &reg.doca_helm_registry, "doca-telemetry", "1.22.1")
    }
}

/// Default DPU services. Used when `config.services` is empty.
pub fn default_services(reg: &ServiceRegistryConfig) -> Vec<ServiceDefinition> {
    vec![dts_service(reg)]
}
