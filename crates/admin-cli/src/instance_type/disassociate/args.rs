/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;
use rpc::forge::RemoveMachineInstanceTypeAssociationRequest;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "Machine Id")]
    pub machine_id: MachineId,
}

impl From<&Args> for RemoveMachineInstanceTypeAssociationRequest {
    fn from(args: &Args) -> Self {
        Self {
            machine_id: args.machine_id.to_string(),
        }
    }
}
