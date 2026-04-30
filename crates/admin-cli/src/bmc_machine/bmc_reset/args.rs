/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge as forgerpc;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, help = "ID of the machine to reboot")]
    pub machine: String,
    #[clap(short, long, help = "Use ipmitool")]
    pub use_ipmitool: bool,
}

impl From<Args> for forgerpc::AdminBmcResetRequest {
    fn from(args: Args) -> Self {
        Self {
            bmc_endpoint_request: None,
            machine_id: Some(args.machine),
            use_ipmitool: args.use_ipmitool,
        }
    }
}
