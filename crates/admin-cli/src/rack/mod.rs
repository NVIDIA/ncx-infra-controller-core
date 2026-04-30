/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod delete;
mod list;
mod maintenance;
pub mod metadata;
pub mod profile;
mod show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Show rack information")]
    Show(show::Args),
    #[clap(about = "List all racks")]
    List(list::Args),
    #[clap(about = "Delete the rack")]
    Delete(delete::Args),
    #[clap(subcommand, about = "Edit Metadata associated with a Rack")]
    Metadata(metadata::Args),
    #[clap(subcommand, about = "Rack profile")]
    Profile(profile::Args),
    #[clap(subcommand, about = "On-demand rack maintenance")]
    Maintenance(maintenance::Args),
}
