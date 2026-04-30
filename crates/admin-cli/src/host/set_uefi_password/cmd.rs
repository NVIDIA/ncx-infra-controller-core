/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn set_uefi_password(data: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let response = api_client.0.set_host_uefi_password(data).await?;
    println!(
        "successfully set UEFI password for host (jid: {:#?})",
        response.job_id
    );
    Ok(())
}
