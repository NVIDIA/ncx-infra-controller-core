/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod args;
pub mod cmd;

use ::rpc::admin_cli::CarbideCliResult;
pub use args::Args;

use super::common::GlobalOptions;
use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

impl Run for Args {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        let opts = GlobalOptions {
            format: ctx.config.format,
            page_size: ctx.config.page_size,
            sort_by: &ctx.config.sort_by,
            cloud_unsafe_op: if ctx.config.cloud_unsafe_op_enabled {
                Some("enabled".to_string())
            } else {
                None
            },
        };
        cmd::update_os(&ctx.api_client, self, opts).await?;
        Ok(())
    }
}
