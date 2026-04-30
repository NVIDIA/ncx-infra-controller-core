/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use ::rpc::forge as forgerpc;

use super::args::Args;
use crate::bmc_machine::common::AdminPowerControlAction;
use crate::rpc::ApiClient;

pub async fn lockdown(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let machine = args.machine;
    let action = if args.enable {
        forgerpc::LockdownAction::Enable
    } else if args.disable {
        forgerpc::LockdownAction::Disable
    } else {
        return Err(CarbideCliError::GenericError(
            "Either --enable or --disable must be specified".to_string(),
        ));
    };

    api_client.lockdown(None, machine, action).await?;

    let action_str = if args.enable { "enabled" } else { "disabled" };

    if args.reboot {
        api_client
            .admin_power_control(
                None,
                Some(machine.to_string()),
                AdminPowerControlAction::ForceRestart.into(),
            )
            .await?;
        println!(
            "Lockdown {} and reboot initiated to apply the change.",
            action_str
        );
    } else {
        println!(
            "Lockdown {}. Please reboot the machine to apply the change.",
            action_str
        );
    }
    Ok(())
}
