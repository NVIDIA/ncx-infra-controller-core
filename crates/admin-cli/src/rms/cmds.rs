/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;

use librms::RmsApi;

use crate::rms::args::{FirmwareInventory, PowerOnSequence, PowerState};

pub async fn get_all_inventory(rms_client: &Arc<dyn RmsApi>) -> eyre::Result<()> {
    let cmd = librms::protos::rack_manager::GetAllInventoryRequest::default();
    let response = rms_client.get_all_inventory(cmd).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

pub async fn power_on_sequence(
    args: PowerOnSequence,
    rms_client: &Arc<dyn RmsApi>,
) -> eyre::Result<()> {
    let response = rms_client.get_rack_power_on_sequence(args.into()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

pub async fn power_state(args: PowerState, rms_client: &Arc<dyn RmsApi>) -> eyre::Result<()> {
    let response = rms_client.get_power_state(args.into()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}

pub async fn get_firmware_inventory(
    args: FirmwareInventory,
    rms_client: &Arc<dyn RmsApi>,
) -> eyre::Result<()> {
    let response = rms_client.get_node_firmware_inventory(args.into()).await?;
    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
