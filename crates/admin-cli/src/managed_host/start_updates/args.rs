/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, required(true), help = "Machine IDs to update, space separated", num_args = 1.., value_delimiter = ' ')]
    pub machines: Vec<MachineId>,
    #[clap(
        long,
        help = "Start of the maintenance window for doing the updates (default now) format 2025-01-02T03:04:05+0000 or 2025-01-02T03:04:05 for local time"
    )]
    pub start: Option<String>,
    #[clap(
        long,
        help = "End of starting new updates (default 24 hours from the start) format 2025-01-02T03:04:05+0000 or 2025-01-02T03:04:05 for local time"
    )]
    pub end: Option<String>,
    #[arg(long, help = "Cancel any new updates")]
    pub cancel: bool,
}
