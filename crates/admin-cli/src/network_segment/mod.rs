/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod delete;
mod show;

// Cross-module re-exports for jump module
pub use show::args::Args as ShowNetworkSegment;
pub use show::cmd::handle_show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Display Network Segment information")]
    Show(show::Args),
    #[clap(about = "Delete Network Segment")]
    Delete(delete::Args),
}
