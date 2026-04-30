/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::BmcEndpointRequest;
use mac_address::MacAddress;

use crate::rpc::ApiClient;

pub async fn explore(
    api_client: &ApiClient,
    address: &str,
    mac: Option<MacAddress>,
) -> CarbideCliResult<()> {
    let report = api_client
        .0
        .explore(BmcEndpointRequest {
            ip_address: address.to_string(),
            mac_address: mac.map(|m| m.to_string()),
        })
        .await?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
