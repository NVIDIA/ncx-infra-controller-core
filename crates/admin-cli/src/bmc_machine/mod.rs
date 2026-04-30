/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
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

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
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
