/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod ensure;
mod show;

// Cross-module re-exports for jump module.
pub use show::args::Args as ShowDpa;
pub use show::cmd::show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Create/ensure a DPA interface")]
    Ensure(ensure::Args),
    #[clap(about = "Display Dpa information")]
    Show(show::Args),
}
