/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;
use rpc::forge as forgerpc;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "ID of the host machine")]
    pub host_machine_id: MachineId,
    #[clap(help = "ID of the DPU machine to make primary")]
    pub dpu_machine_id: MachineId,
    #[clap(long, help = "Reboot the host after the update")]
    pub reboot: bool,
}

impl From<Args> for forgerpc::SetPrimaryDpuRequest {
    fn from(args: Args) -> Self {
        Self {
            host_machine_id: Some(args.host_machine_id),
            dpu_machine_id: Some(args.dpu_machine_id),
            reboot: args.reboot,
        }
    }
}
