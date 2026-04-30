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
            Args::Show(args) => {
                cmd::power_options_show(args, ctx.config.format, &ctx.api_client).await?
            }
            Args::Update(args) => cmd::update_power_option(args, &ctx.api_client).await?,
            Args::GetMachineIngestionState(mac_address) => {
                cmd::get_machine_state(&ctx.api_client, &mac_address.mac_address).await?
            }
            Args::AllowIngestionAndPowerOn(mac_address) => {
                cmd::allow_ingestion_and_power_on(&ctx.api_client, &mac_address.mac_address).await?
            }
        }
        Ok(())
    }
}
