/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod args;
pub mod cmds;

#[cfg(test)]
mod tests;

use ::rpc::admin_cli::CarbideCliResult;
pub use args::Cmd;

use crate::cfg::dispatch::Dispatch;
use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

impl Run for Cmd {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        cmds::jump(self, ctx)
            .await
            .map_err(|e| ::rpc::admin_cli::CarbideCliError::GenericError(e.to_string()))
    }
}

impl Dispatch for Cmd {
    async fn dispatch(self, mut ctx: RuntimeContext) -> CarbideCliResult<()> {
        self.run(&mut ctx).await
    }
}
