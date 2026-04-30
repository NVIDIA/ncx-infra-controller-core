/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge::AdminForceDeleteMachineRequest;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(
        long,
        help = "UUID, IPv4, MAC or hostnmame of the host or DPU machine to delete"
    )]
    pub machine: String,

    #[clap(
        short = 'd',
        long,
        action,
        help = "Delete interfaces. Redeploy kea after deleting machine interfaces."
    )]
    pub delete_interfaces: bool,

    #[clap(
        short = 'b',
        long,
        action,
        help = "Delete BMC interfaces. Redeploy kea after deleting machine interfaces."
    )]
    pub delete_bmc_interfaces: bool,

    #[clap(
        short = 'c',
        long,
        action,
        help = "Delete BMC credentials. Only applicable if site explorer has configured credentials for the BMCs associated with this managed host."
    )]
    pub delete_bmc_credentials: bool,

    #[clap(
        long,
        action,
        help = "Delete machine with allocated instance. This flag acknowledges destroying the user instance as well."
    )]
    pub allow_delete_with_instance: bool,
}

impl From<&Args> for AdminForceDeleteMachineRequest {
    fn from(args: &Args) -> Self {
        Self {
            host_query: args.machine.clone(),
            delete_interfaces: args.delete_interfaces,
            delete_bmc_interfaces: args.delete_bmc_interfaces,
            delete_bmc_credentials: args.delete_bmc_credentials,
        }
    }
}
