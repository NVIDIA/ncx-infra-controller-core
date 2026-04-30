/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use crate::bmc_machine::common::{AdminPowerControlAction, InfiniteBootArgs};
use crate::rpc::ApiClient;

pub async fn enable_infinite_boot(
    args: InfiniteBootArgs,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let machine = args.machine;
    api_client
        .enable_infinite_boot(None, Some(machine.clone()))
        .await?;
    if args.reboot {
        api_client
            .admin_power_control(
                None,
                Some(machine),
                AdminPowerControlAction::ForceRestart.into(),
            )
            .await?;
    }
    Ok(())
}
