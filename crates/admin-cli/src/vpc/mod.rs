/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod set_virtualizer;
mod show;

// Cross-module re-exports for jump module
pub use show::args::Args as ShowVpc;
pub use show::cmd::show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Display VPC information")]
    Show(show::Args),
    SetVirtualizer(set_virtualizer::Args),
}
