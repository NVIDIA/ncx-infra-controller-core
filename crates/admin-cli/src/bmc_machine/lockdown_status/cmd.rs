/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn lockdown_status(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let response = api_client.0.lockdown_status(args).await?;
    // Convert status enum to string
    let status_str = match response.status {
        0 => "Enabled",  // InternalLockdownStatus::ENABLED
        1 => "Partial",  // InternalLockdownStatus::PARTIAL
        2 => "Disabled", // InternalLockdownStatus::DISABLED
        _ => "Unknown",
    };
    println!("{}: {}", status_str, response.message);
    Ok(())
}
