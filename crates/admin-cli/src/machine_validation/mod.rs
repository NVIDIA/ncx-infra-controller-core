/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod external_config;
mod on_demand;
mod results;
mod runs;
mod tests_cmd;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "External config", subcommand, visible_alias = "mve")]
    ExternalConfig(external_config::Args),
    #[clap(about = "Ondemand Validation", subcommand, visible_alias = "mvo")]
    OnDemand(on_demand::Args),
    #[clap(
        about = "Display machine validation results of individual runs",
        subcommand,
        visible_alias = "mvr"
    )]
    Results(results::Args),
    #[clap(
        about = "Display all machine validation runs",
        subcommand,
        visible_alias = "mvt"
    )]
    Runs(runs::Args),
    #[clap(about = "Supported Tests ", subcommand, visible_alias = "mvs")]
    Tests(tests_cmd::Args),
}
