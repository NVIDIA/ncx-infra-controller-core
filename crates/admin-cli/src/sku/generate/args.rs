/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The machine id of the machine to use to generate a SKU")]
    pub machine_id: MachineId,
    #[clap(help = "override the ID of the SKU", long)]
    pub id: Option<String>,
}
