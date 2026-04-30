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
        if self.has_duplicate_dpu_serials() {
            eprintln!("Duplicate values not allowed for --fallback-dpu-serial-number");
            return Ok(());
        }
        let expected_machine: rpc::forge::ExpectedMachine = self.try_into()?;
        ctx.api_client
            .0
            .add_expected_machine(expected_machine)
            .await?;
        Ok(())
    }
}
