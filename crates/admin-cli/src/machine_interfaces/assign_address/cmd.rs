/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge as forgerpc;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn handle_assign_address(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let resp = api_client
        .0
        .assign_static_address(forgerpc::AssignStaticAddressRequest {
            interface_id: Some(args.interface_id),
            ip_address: args.ip_address.to_string(),
        })
        .await?;

    match resp.status() {
        forgerpc::AssignStaticAddressStatus::Assigned => {
            println!(
                "Assigned static address {} to interface {}",
                resp.ip_address, args.interface_id
            );
        }
        forgerpc::AssignStaticAddressStatus::ReplacedStatic => {
            println!(
                "Replaced existing static address with {} on interface {}",
                resp.ip_address, args.interface_id
            );
        }
        forgerpc::AssignStaticAddressStatus::ReplacedDhcp => {
            println!(
                "Replaced DHCP allocation with static address {} on interface {}",
                resp.ip_address, args.interface_id
            );
        }
    }

    Ok(())
}
