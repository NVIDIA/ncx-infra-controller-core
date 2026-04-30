/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use crate::rpc::ApiClient;

pub async fn show(api_client: &ApiClient) -> CarbideCliResult<()> {
    let ca_certs = api_client.0.tpm_show_ca_certs().await?.tpm_ca_cert_details;
    println!("{}", serde_json::to_string_pretty(&ca_certs)?);

    Ok(())
}
