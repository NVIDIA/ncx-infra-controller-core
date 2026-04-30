/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod clear_uefi_password;
mod generate_host_uefi_password;
mod reprovision;
mod set_uefi_password;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Set Host UEFI password")]
    SetUefiPassword(set_uefi_password::Args),
    #[clap(about = "Clear Host UEFI password")]
    ClearUefiPassword(clear_uefi_password::Args),
    #[clap(about = "Generates a string that can be a site-default host UEFI password in Vault")]
    /// - the generated string will meet the uefi password requirements of all vendors
    GenerateHostUefiPassword(generate_host_uefi_password::Args),
    #[clap(subcommand, about = "Host reprovisioning handling")]
    Reprovision(reprovision::Args),
}
