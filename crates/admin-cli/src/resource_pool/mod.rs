/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod grow;
mod list;

// Cross-module re-export for jump module
pub use list::cmd::list;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(
        about = "Add capacity to one or more resource pools from a TOML file. See carbide-api admin_grow_resource_pool docs for example TOML."
    )]
    Grow(grow::Args),
    #[clap(about = "List all resource pools with stats")]
    List(list::Args),
}
