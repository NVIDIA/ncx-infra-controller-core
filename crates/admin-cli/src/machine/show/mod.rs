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
        cmd::handle_show(
            self,
            &ctx.config.format,
            &mut ctx.output_file,
            &ctx.api_client,
            ctx.config.page_size,
            &ctx.config.sort_by,
        )
        .await?;
        Ok(())
    }
}
