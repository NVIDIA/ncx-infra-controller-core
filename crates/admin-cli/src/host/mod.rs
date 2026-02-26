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

mod clear_uefi_password;
mod generate_host_uefi_password;
mod reprovision;
mod set_uefi_password;

#[cfg(test)]
mod tests;

use ::rpc::admin_cli::CarbideCliResult;
use clap::Parser;

use crate::cfg::dispatch::Dispatch;
use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Set Host UEFI password")]
    SetUefiPassword(set_uefi_password::Args),
    #[clap(about = "Clear Host UEFI password")]
    ClearUefiPassword(clear_uefi_password::Args),
    #[clap(about = "Generates a string that can be a site-default host UEFI password in Vault")]
    /// - the generated string will meet the uefi password requirements of all vendors
    GenerateHostUefiPassword(generate_host_uefi_password::Args),
    #[clap(subcommand, about = "Host reprovisioning handling")]
    Reprovision(reprovision::Args),
}

impl Dispatch for Cmd {
    async fn dispatch(self, mut ctx: RuntimeContext) -> CarbideCliResult<()> {
        match self {
            Cmd::SetUefiPassword(args) => args.run(&mut ctx).await,
            Cmd::ClearUefiPassword(args) => args.run(&mut ctx).await,
            Cmd::GenerateHostUefiPassword(args) => args.run(&mut ctx).await,
            Cmd::Reprovision(args) => args.run(&mut ctx).await,
        }
    }
}
