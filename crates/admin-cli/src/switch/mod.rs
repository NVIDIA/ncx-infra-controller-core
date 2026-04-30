/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod force_delete;
pub mod health_report;
mod list;
pub mod metadata;
mod show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Show switch information")]
    Show(show::Args),
    #[clap(about = "List all switches")]
    List(list::Args),
    #[clap(about = "Force delete a switch and optionally its interfaces")]
    ForceDelete(force_delete::Args),
    #[clap(subcommand, about = "Manage Switch Metadata")]
    Metadata(metadata::Args),
    #[dispatch]
    #[clap(
        about = "Manage health report sources",
        subcommand,
        visible_alias = "hr"
    )]
    HealthReport(health_report::Args),
}
