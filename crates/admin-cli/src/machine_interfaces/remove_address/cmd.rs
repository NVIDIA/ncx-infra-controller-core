/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge as forgerpc;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn handle_remove_address(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let resp = api_client
        .0
        .remove_static_address(forgerpc::RemoveStaticAddressRequest {
            interface_id: Some(args.interface_id),
            ip_address: args.ip_address.to_string(),
        })
        .await?;

    match resp.status() {
        forgerpc::RemoveStaticAddressStatus::Removed => {
            println!(
                "Removed static address {} from interface {}",
                resp.ip_address, args.interface_id
            );
        }
        forgerpc::RemoveStaticAddressStatus::NotFound => {
            println!(
                "No static address {} found on interface {}",
                resp.ip_address, args.interface_id
            );
        }
    }

    Ok(())
}
