/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use crate::rpc::ApiClient;

pub async fn show_unmatched_ek(api_client: &ApiClient) -> CarbideCliResult<()> {
    let unmatched_eks = api_client
        .0
        .tpm_show_unmatched_ek_certs()
        .await?
        .tpm_ek_cert_statuses;
    println!("{}", serde_json::to_string_pretty(&unmatched_eks)?);

    Ok(())
}
