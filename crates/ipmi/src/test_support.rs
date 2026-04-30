/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::IpAddr;

use async_trait::async_trait;
use carbide_uuid::machine::MachineId;
use forge_secrets::credentials::CredentialKey;

use crate::IPMITool;

pub struct IPMIToolTestImpl {}

#[async_trait]
impl IPMITool for IPMIToolTestImpl {
    async fn restart(
        &self,
        _machine_id: &MachineId,
        _bmc_ip: IpAddr,
        _legacy_boot: bool,
        _credential_key: &CredentialKey,
    ) -> Result<(), eyre::Report> {
        Ok(())
    }

    async fn bmc_cold_reset(
        &self,
        _bmc_ip: IpAddr,
        _credential_key: &CredentialKey,
    ) -> Result<(), eyre::Report> {
        Ok(())
    }
}
