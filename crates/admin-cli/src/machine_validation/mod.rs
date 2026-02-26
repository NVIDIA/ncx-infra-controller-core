/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

mod external_config;
mod on_demand;
mod results;
mod runs;
mod tests_cmd;

#[cfg(test)]
mod tests;

use ::rpc::admin_cli::CarbideCliResult;
use clap::Parser;

use crate::cfg::dispatch::Dispatch;
use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

#[derive(Parser, Debug)]
pub enum Cmd {
    #[clap(about = "External config", subcommand, visible_alias = "mve")]
    ExternalConfig(external_config::Args),
    #[clap(about = "Ondemand Validation", subcommand, visible_alias = "mvo")]
    OnDemand(on_demand::Args),
    #[clap(
        about = "Display machine validation results of individual runs",
        subcommand,
        visible_alias = "mvr"
    )]
    Results(results::Args),
    #[clap(
        about = "Display all machine validation runs",
        subcommand,
        visible_alias = "mvt"
    )]
    Runs(runs::Args),
    #[clap(about = "Supported Tests ", subcommand, visible_alias = "mvs")]
    Tests(Box<tests_cmd::Args>),
}

impl Dispatch for Cmd {
    async fn dispatch(self, mut ctx: RuntimeContext) -> CarbideCliResult<()> {
        match self {
            Cmd::ExternalConfig(cmd) => cmd.run(&mut ctx).await,
            Cmd::Results(cmd) => cmd.run(&mut ctx).await,
            Cmd::Runs(cmd) => cmd.run(&mut ctx).await,
            Cmd::OnDemand(cmd) => cmd.run(&mut ctx).await,
            Cmd::Tests(cmd) => (*cmd).run(&mut ctx).await,
        }
    }
}
