/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn reboot(api_client: &ApiClient, args: Args) -> CarbideCliResult<()> {
    let res = api_client
        .admin_power_control(
            None,
            Some(args.machine),
            ::rpc::forge::admin_power_control_request::SystemPowerControl::ForceRestart,
        )
        .await?;

    if let Some(msg) = res.msg {
        println!("{msg}");
    }
    Ok(())
}
