/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct MachineQuery {
    #[clap(
        short,
        long,
        help = "ID, IPv4, MAC or hostnmame of the machine to query"
    )]
    pub query: String,
}

#[derive(Parser, Debug, Clone)]
pub struct NetworkConfigQuery {
    #[clap(long, required(true), help = "DPU machine id")]
    pub machine_id: MachineId,
}
