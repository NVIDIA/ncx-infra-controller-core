/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;
use clap::Parser;
use mac_address::MacAddress;
use uuid::Uuid;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "BMC MAC address of the expected machine to delete.")]
    pub bmc_mac_address: Option<MacAddress>,

    #[clap(long, help = "ID (UUID) of the expected machine to delete.")]
    pub id: Option<Uuid>,
}

impl TryFrom<Args> for ::rpc::forge::ExpectedMachineRequest {
    type Error = CarbideCliError;
    fn try_from(args: Args) -> Result<Self, Self::Error> {
        match (args.bmc_mac_address, args.id) {
            (Some(_), Some(_)) => Err(CarbideCliError::ChooseOneError("--bmc-mac-address", "--id")),
            (None, None) => Err(CarbideCliError::RequireOneError(
                "--bmc-mac-address",
                "--id",
            )),
            (None, Some(id)) => Ok(Self {
                bmc_mac_address: String::new(),
                id: Some(::rpc::common::Uuid {
                    value: id.to_string(),
                }),
            }),
            (Some(mac), None) => Ok(Self {
                bmc_mac_address: mac.to_string(),
                id: None,
            }),
        }
    }
}
