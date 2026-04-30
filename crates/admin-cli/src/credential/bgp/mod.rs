/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod delete_sitewide;
mod set_sitewide;

use ::rpc::admin_cli::CarbideCliResult;
use clap::Parser;

use crate::cfg::dispatch::Dispatch;
use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(name = "set-sitewide", about = "Set the site-wide leaf BGP password")]
    Set(set_sitewide::Args),
    #[clap(
        name = "delete-sitewide",
        about = "Delete the site-wide leaf BGP password"
    )]
    Delete(delete_sitewide::Args),
}

impl Run for Cmd {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        match self {
            Cmd::Set(args) => args.run(ctx).await,
            Cmd::Delete(args) => args.run(ctx).await,
        }
    }
}
