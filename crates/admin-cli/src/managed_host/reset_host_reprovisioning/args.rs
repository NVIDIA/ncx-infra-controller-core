/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

/// Reset host reprovisioning state
#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, required(true), help = "Machine ID to reset host reprovision on")]
    pub machine: MachineId,
}
