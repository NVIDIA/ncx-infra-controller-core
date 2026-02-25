/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

mod admin_power_control;
mod bmc_reset;
pub(crate) mod common;
mod create_bmc_user;
mod delete_bmc_user;
mod enable_infinite_boot;
mod is_infinite_boot_enabled;
mod lockdown;
mod lockdown_status;

#[cfg(test)]
mod tests;

use ::rpc::admin_cli::CarbideCliResult;
use clap::Parser;

use crate::cfg::dispatch::Dispatch;
use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Reset BMC")]
    BmcReset(bmc_reset::Args),
    #[clap(about = "Redfish Power Control")]
    AdminPowerControl(admin_power_control::Args),
    CreateBmcUser(create_bmc_user::Args),
    DeleteBmcUser(delete_bmc_user::Args),
    #[clap(about = "Enable infinite boot")]
    EnableInfiniteBoot(enable_infinite_boot::Args),
    #[clap(about = "Check if infinite boot is enabled")]
    IsInfiniteBootEnabled(is_infinite_boot_enabled::Args),
    #[clap(about = "Enable or disable lockdown")]
    Lockdown(lockdown::Args),
    #[clap(about = "Check lockdown status")]
    LockdownStatus(lockdown_status::Args),
}

impl Dispatch for Cmd {
    async fn dispatch(self, mut ctx: RuntimeContext) -> CarbideCliResult<()> {
        match self {
            Cmd::BmcReset(args) => args.run(&mut ctx).await,
            Cmd::AdminPowerControl(args) => args.run(&mut ctx).await,
            Cmd::CreateBmcUser(args) => args.run(&mut ctx).await,
            Cmd::DeleteBmcUser(args) => args.run(&mut ctx).await,
            Cmd::EnableInfiniteBoot(args) => args.run(&mut ctx).await,
            Cmd::IsInfiniteBootEnabled(args) => args.run(&mut ctx).await,
            Cmd::Lockdown(args) => args.run(&mut ctx).await,
            Cmd::LockdownStatus(args) => args.run(&mut ctx).await,
        }
    }
}
