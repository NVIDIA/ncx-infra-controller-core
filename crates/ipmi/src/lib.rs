/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::IpAddr;
use std::sync::Arc;

use arc_swap::ArcSwap;
use async_trait::async_trait;
use carbide_utils::HostPortPair;
use carbide_uuid::machine::MachineId;
use forge_secrets::credentials::{CredentialKey, CredentialReader};

mod bmc_mock;
mod test_support;
mod tool;

#[async_trait]
pub trait IPMITool: Send + Sync + 'static {
    async fn bmc_cold_reset(
        &self,
        bmc_ip: IpAddr,
        credential_key: &CredentialKey,
    ) -> Result<(), eyre::Report>;

    async fn restart(
        &self,
        machine_id: &MachineId,
        bmc_ip: IpAddr,
        legacy_boot: bool,
        credential_key: &CredentialKey,
    ) -> Result<(), eyre::Report>;
}

pub fn tool(cred_provider: Arc<dyn CredentialReader>, attempts: Option<u32>) -> Arc<dyn IPMITool> {
    Arc::new(tool::IPMIToolImpl::new(cred_provider, attempts))
}

pub fn bmc_mock(bmc_proxy: Arc<ArcSwap<Option<HostPortPair>>>) -> Arc<dyn IPMITool> {
    Arc::new(bmc_mock::IPMIToolHttpImpl::new(bmc_proxy))
}

pub fn test_support() -> Arc<dyn IPMITool> {
    Arc::new(test_support::IPMIToolTestImpl {})
}
