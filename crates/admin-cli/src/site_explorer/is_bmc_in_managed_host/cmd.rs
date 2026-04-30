/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::BmcEndpointRequest;
use mac_address::MacAddress;

use crate::rpc::ApiClient;

pub async fn is_bmc_in_managed_host(
    api_client: &ApiClient,
    address: &str,
    mac: Option<MacAddress>,
) -> CarbideCliResult<()> {
    let is_bmc_in_managed_host = api_client
        .0
        .is_bmc_in_managed_host(BmcEndpointRequest {
            ip_address: address.to_string(),
            mac_address: mac.map(|m| m.to_string()),
        })
        .await?;
    println!(
        "Is {} in a managed host?: {}",
        address, is_bmc_in_managed_host.in_managed_host
    );
    Ok(())
}
