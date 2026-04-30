/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::ConfigSetting;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn create_machines(opts: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let enabled = opts.is_enabled();
    api_client
        .set_dynamic_config(ConfigSetting::CreateMachines, enabled.to_string(), None)
        .await?;
    let state = if enabled { "enabled" } else { "disabled" };
    println!("create-machines {state}");
    Ok(())
}
