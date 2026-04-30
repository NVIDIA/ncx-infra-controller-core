/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn clear_uefi_password(data: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let response = api_client.0.clear_host_uefi_password(data).await?;
    println!(
        "successfully cleared UEFI password for host (jid: {:#?})",
        response.job_id
    );
    Ok(())
}
