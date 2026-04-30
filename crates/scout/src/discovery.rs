/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge as rpc;
use carbide_uuid::machine::MachineId;

use crate::CarbideClientError;
use crate::cfg::Options;
use crate::client::create_forge_client;

pub(crate) async fn completed(
    config: &Options,
    machine_id: &MachineId,
) -> Result<(), CarbideClientError> {
    let mut client = create_forge_client(config).await?;
    let request = tonic::Request::new(rpc::MachineDiscoveryCompletedRequest {
        machine_id: Some(*machine_id),
    });
    client.discovery_completed(request).await?;
    Ok(())
}
pub(crate) async fn rebooted(
    config: &Options,
    machine_id: &MachineId,
) -> Result<(), CarbideClientError> {
    let mut client = create_forge_client(config).await?;
    let request = tonic::Request::new(rpc::MachineRebootCompletedRequest {
        machine_id: Some(*machine_id),
    });
    client.reboot_completed(request).await?;
    Ok(())
}
