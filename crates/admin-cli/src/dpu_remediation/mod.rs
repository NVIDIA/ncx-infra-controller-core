/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod approve;
mod create;
mod disable;
mod enable;
mod list_applied;
mod revoke;
mod show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Create a remediation")]
    Create(create::Args),
    #[clap(about = "Approve a remediation")]
    Approve(approve::Args),
    #[clap(about = "Revoke a remediation")]
    Revoke(revoke::Args),
    #[clap(about = "Enable a remediation")]
    Enable(enable::Args),
    #[clap(about = "Disable a remediation")]
    Disable(disable::Args),
    #[clap(about = "Display remediation information")]
    Show(show::Args),
    #[clap(about = "Display information about applied remediations")]
    ListApplied(list_applied::Args),
}
