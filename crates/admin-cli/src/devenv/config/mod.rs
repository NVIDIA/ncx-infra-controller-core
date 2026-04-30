/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod apply;

use ::rpc::admin_cli::CarbideCliResult;
#[cfg(test)]
pub use apply::args::NetworkChoice;
use clap::Parser;

use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

#[derive(Parser, Debug, Clone)]
pub enum Cmd {
    #[clap(about = "Apply devenv config", visible_alias = "a")]
    Apply(apply::Args),
}

impl Run for Cmd {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        match self {
            Cmd::Apply(args) => args.run(ctx).await,
        }
    }
}
