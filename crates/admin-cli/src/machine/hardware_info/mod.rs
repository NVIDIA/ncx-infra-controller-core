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
            Args::Show(show_cmd) => {
                cmd::handle_show_machine_hardware_info(
                    &ctx.api_client,
                    &mut ctx.output_file,
                    &ctx.config.format,
                    show_cmd.machine,
                )?;
            }
            Args::Update(capability) => match capability {
                args::MachineHardwareInfo::Gpus(gpus) => {
                    cmd::handle_update_machine_hardware_info_gpus(&ctx.api_client, gpus).await?;
                }
            },
        }
        Ok(())
    }
}
