/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod assign_address;
mod delete;
mod remove_address;
mod show;
mod show_addresses;

// Cross-module re-exports for jump module
pub use show::args::Args as ShowMachineInterfaces;
pub use show::cmd::handle_show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "List of all Machine interfaces")]
    Show(show::Args),
    #[clap(about = "Delete Machine interface.")]
    Delete(delete::Args),
    #[clap(about = "Show addresses for a machine interface")]
    ShowAddresses(show_addresses::Args),
    #[clap(about = "Assign a static address to a machine interface")]
    AssignAddress(assign_address::Args),
    #[clap(about = "Remove a static address from a machine interface")]
    RemoveAddress(remove_address::Args),
}
