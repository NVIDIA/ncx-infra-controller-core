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
            Args::Set(data) => cmd::trigger_reprovisioning_set(data, &ctx.api_client).await,
            Args::Clear(data) => cmd::trigger_reprovisioning_clear(data, &ctx.api_client).await,
            Args::List => cmd::list_hosts_pending(&ctx.api_client).await,
            Args::MarkManualUpgradeComplete(data) => {
                cmd::mark_manual_firmware_upgrade_complete(data.id, &ctx.api_client).await
            }
        }
    }
}
