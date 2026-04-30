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
            Args::Show(options) => {
                cmd::show_tests(
                    &ctx.api_client,
                    options,
                    ctx.config.format,
                    ctx.config.extended,
                )
                .await
            }
            Args::Verify(options) => {
                cmd::machine_validation_test_verfied(&ctx.api_client, options).await
            }
            Args::Enable(options) => {
                cmd::machine_validation_test_enable(&ctx.api_client, options).await
            }
            Args::Disable(options) => {
                cmd::machine_validation_test_disable(&ctx.api_client, options).await
            }
            Args::Add(options) => cmd::machine_validation_test_add(&ctx.api_client, options).await,
            Args::Update(options) => {
                cmd::machine_validation_test_update(&ctx.api_client, options).await
            }
        }
    }
}
