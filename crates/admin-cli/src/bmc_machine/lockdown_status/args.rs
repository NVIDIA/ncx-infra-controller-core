/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;
use rpc::forge as forgerpc;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, help = "ID of the machine to check lockdown status")]
    pub machine: MachineId,
}

impl From<Args> for forgerpc::LockdownStatusRequest {
    fn from(args: Args) -> Self {
        Self {
            bmc_endpoint_request: None,
            machine_id: Some(args.machine),
        }
    }
}
