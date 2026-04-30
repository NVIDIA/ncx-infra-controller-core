/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use ::rpc::forge::MachineBootOverride;
use carbide_uuid::machine::MachineInterfaceId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    pub interface_id: MachineInterfaceId,
    #[clap(short = 'p', long)]
    pub custom_pxe: Option<String>,
    #[clap(short = 'u', long)]
    pub custom_user_data: Option<String>,
}

impl TryFrom<Args> for MachineBootOverride {
    type Error = CarbideCliError;

    fn try_from(args: Args) -> CarbideCliResult<Self> {
        if args.custom_pxe.is_none() && args.custom_user_data.is_none() {
            return Err(CarbideCliError::GenericError(
                "Either custom pxe or custom user data is required".to_owned(),
            ));
        }

        let custom_pxe = args.custom_pxe.map(std::fs::read_to_string).transpose()?;
        let custom_user_data = args
            .custom_user_data
            .map(std::fs::read_to_string)
            .transpose()?;

        Ok(MachineBootOverride {
            machine_interface_id: Some(args.interface_id),
            custom_pxe,
            custom_user_data,
        })
    }
}
