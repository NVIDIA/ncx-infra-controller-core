/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod agent_upgrade_policy;
mod network;
mod reprovision;
mod status;
mod versions;

// Cross-module re-exports for machine module
pub use network::cmd::show_dpu_status;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(subcommand, about = "DPU Reprovisioning handling")]
    Reprovision(reprovision::Args),
    #[clap(about = "Get or set forge-dpu-agent upgrade policy")]
    AgentUpgradePolicy(agent_upgrade_policy::Args),
    #[clap(about = "View DPU firmware status")]
    Versions(versions::Args),
    #[clap(about = "View DPU Status")]
    Status(status::Args),
    #[clap(subcommand, about = "Networking information")]
    Network(network::Args),
}
