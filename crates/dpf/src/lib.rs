/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! # Carbide DPF SDK
//!
//! This crate provides a high-level SDK for interacting with the NVIDIA DPF
//! (DOCA Platform Framework) operator via Kubernetes CRDs.
//!
//! ## Overview
//!
//! The DPF SDK abstracts away the complexity of managing DPF CRDs, providing
//! a clean interface for:
//!
//! - Initializing DPF resources (BFB, DPUFlavor, DPUDeployment with services)
//! - Registering and managing DPU devices
//! - Registering and managing DPU nodes (hosts with DPUs)
//! - Watching for DPF events via callbacks
//!
//! ## Example
//!
//! ```rust,ignore
//! use dpf::{DpfSdkBuilder, KubeRepository, InitDpfResourcesConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let repo = KubeRepository::new().await?;
//!     let config = InitDpfResourcesConfig::default();
//!     let sdk = DpfSdkBuilder::new(repo, "dpf-operator-system", "secret".to_string())
//!         .initialize(&config)
//!         .await?;
//!
//!     let _watcher = sdk.watcher()
//!         .on_reboot_required(|event| async move {
//!             println!("Reboot required for: {}", event.host_bmc_ip);
//!             Ok(())
//!         })
//!         .start()?;
//!
//!     Ok(())
//! }
//! ```
#![warn(clippy::all)]
#![deny(warnings, unsafe_code)]

pub mod crds;
pub mod error;
pub mod flavor;
pub mod repository;
pub mod sdk;
pub mod services;
pub mod types;
pub mod watcher;

#[cfg(test)]
mod test;

// Re-exports for convenience
pub use error::DpfError;
pub use repository::{DpfRepository, KubeRepository};
pub use sdk::{
    DpfSdk, DpfSdkBuilder, NoLabels, ResourceLabeler, build_deployment,
    build_service_configuration, build_service_interface, build_service_nad,
    build_service_template, dpu_cr_name, dpu_device_cr_name, dpu_node_cr_name,
    node_id_from_dpu_node_cr_name,
};
pub use services::{DEFAULT_DOCA_HELM_REGISTRY, ServiceRegistryConfig};
pub use types::{
    BmcPasswordProvider, ConfigPortsServiceType, DpuDeviceInfo, DpuErrorEvent, DpuEvent,
    DpuNodeInfo, DpuPhase, DpuReadyEvent, InitDpfResourcesConfig, MaintenanceEvent,
    RebootRequiredEvent, ServiceChainSwitch, ServiceConfigPort, ServiceConfigPortProtocol,
    ServiceDefinition, ServiceInterface, ServiceNAD, ServiceNADResourceType,
};
pub use watcher::{DpuWatcher, DpuWatcherBuilder};

/// Default namespace for DPF operator resources.
pub const NAMESPACE: &str = "dpf-operator-system";
