/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn is_infinite_boot_enabled(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let response = api_client.0.is_infinite_boot_enabled(args).await?;
    match response.is_enabled {
        Some(true) => println!("Enabled"),
        Some(false) => println!("Disabled"),
        None => println!("Unknown"),
    }
    Ok(())
}
