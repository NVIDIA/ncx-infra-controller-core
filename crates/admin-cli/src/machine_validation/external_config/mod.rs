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
        match self {
            Args::Show(opts) => {
                cmd::external_config_show(
                    &ctx.api_client,
                    opts.name,
                    ctx.config.extended,
                    ctx.config.format,
                )
                .await
            }
            Args::AddUpdate(opts) => {
                cmd::external_config_add_update(
                    &ctx.api_client,
                    opts.name,
                    opts.file_name,
                    opts.description,
                )
                .await
            }
            Args::Remove(opts) => cmd::remove_external_config(&ctx.api_client, opts.name).await,
        }
    }
}
