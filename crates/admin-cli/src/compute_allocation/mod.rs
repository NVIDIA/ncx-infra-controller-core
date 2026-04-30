/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod common;
mod create;
mod delete;
mod show;
mod update;

// Cross-module re-exports for jump module
use clap::Parser;
pub use show::args::Args as ShowComputeAllocation;
pub use show::cmd::show;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Create a compute allocation", visible_alias = "c")]
    Create(create::Args),

    #[clap(about = "Show one or more compute allocations", visible_alias = "s")]
    Show(show::Args),

    #[clap(about = "Delete a compute allocation", visible_alias = "d")]
    Delete(delete::Args),

    #[clap(about = "Update a compute allocation", visible_alias = "u")]
    Update(update::Args),
}
