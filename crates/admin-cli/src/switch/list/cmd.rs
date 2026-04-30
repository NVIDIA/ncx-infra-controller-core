/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use color_eyre::Result;
use rpc::forge as rpc;

use super::args::Args;
use crate::cfg::runtime::RuntimeConfig;
use crate::rpc::ApiClient;

pub async fn list_switches(
    args: Args,
    api_client: &ApiClient,
    config: &RuntimeConfig,
) -> Result<()> {
    let filter = rpc::SwitchSearchFilter {
        rack_id: None,
        deleted: args.deleted as i32,
        controller_state: args.controller_state,
        bmc_mac: args.bmc_mac.map(|m| m.to_string()),
        nvos_mac: args.nvos_mac.map(|m| m.to_string()),
    };
    let response = api_client
        .get_all_switches(filter, config.page_size)
        .await?;

    let switches = response.switches;

    if switches.is_empty() {
        println!("No switches found.");
        return Ok(());
    }

    println!("Found {} switch(es):", switches.len());

    for (i, switch) in switches.iter().enumerate() {
        let name = switch
            .config
            .as_ref()
            .map(|config| config.name.as_str())
            .unwrap_or_else(|| "Unnamed");

        let id = switch
            .id
            .as_ref()
            .map(|id| Cow::Owned(id.to_string()))
            .unwrap_or_else(|| Cow::Borrowed("N/A"));

        let power_state = switch
            .status
            .as_ref()
            .and_then(|status| status.power_state.as_deref())
            .unwrap_or("Unknown");

        let health = switch
            .status
            .as_ref()
            .and_then(|status| status.health_status.as_deref())
            .unwrap_or("Unknown");

        let controller_state = switch.controller_state.as_str();

        let slot_number = switch
            .placement_in_rack
            .as_ref()
            .and_then(|p| p.slot_number)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let tray_index = switch
            .placement_in_rack
            .as_ref()
            .and_then(|p| p.tray_index)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "{}. {} (ID: {}) - Slot: {}, Tray: {}, Power: {}, Health: {}, State: {}",
            i + 1,
            name,
            id,
            slot_number,
            tray_index,
            power_state,
            health,
            controller_state
        );
    }

    Ok(())
}
