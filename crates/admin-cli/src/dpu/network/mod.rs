/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod args;
pub mod cmd;

use ::rpc::admin_cli::CarbideCliResult;
pub use args::Args;

use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

impl Run for Args {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        let cmd = match self {
            Args::Status => crate::machine::network::Args::Status,
            Args::Config(q) => crate::machine::network::Args::Config(q),
        };
        cmd::network(
            &ctx.api_client,
            &mut ctx.output_file,
            cmd,
            ctx.config.format,
        )
        .await
    }
}
