/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod debug_bundle;
mod maintenance;
mod power_options;
mod quarantine;
mod reset_host_reprovisioning;
mod set_primary_dpu;
mod show;
mod start_updates;

// Cross-module re-exports for firmware/start_updates.rs
// Cross-module re-exports for debug_bundle/cmds.rs
pub use debug_bundle::args::Args as DebugBundle;
pub use start_updates::args::Args as StartUpdates;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Display managed host information")]
    Show(show::Args),
    #[clap(
        about = "Switch a machine in/out of maintenance mode",
        subcommand,
        visible_alias = "fix"
    )]
    Maintenance(maintenance::Args),
    #[clap(
        about = "Quarantine a host (disabling network access on host)",
        subcommand
    )]
    Quarantine(quarantine::Args),
    #[clap(about = "Reset host reprovisioning back to CheckingFirmware")]
    ResetHostReprovisioning(reset_host_reprovisioning::Args),
    #[clap(subcommand, about = "Power Manager related settings.")]
    PowerOptions(power_options::Args),
    #[clap(about = "Start updates for machines with delayed updates, such as GB200")]
    StartUpdates(start_updates::Args),
    #[clap(about = "Set the primary DPU for the managed host")]
    SetPrimaryDpu(set_primary_dpu::Args),
    #[clap(about = "Download debug bundle with logs for a specific host")]
    DebugBundle(debug_bundle::Args),
}
