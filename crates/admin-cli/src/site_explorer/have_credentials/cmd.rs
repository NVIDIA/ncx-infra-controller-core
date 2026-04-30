/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::BmcEndpointRequest;
use mac_address::MacAddress;

use crate::rpc::ApiClient;

pub async fn have_credentials(
    api_client: &ApiClient,
    address: &str,
    mac: Option<MacAddress>,
) -> CarbideCliResult<()> {
    let have_credentials = api_client
        .0
        .bmc_credential_status(BmcEndpointRequest {
            ip_address: address.to_string(),
            mac_address: mac.map(|m| m.to_string()),
        })
        .await?;
    println!("{}", have_credentials.have_credentials);
    Ok(())
}
