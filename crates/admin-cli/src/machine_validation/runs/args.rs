/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Args {
    #[clap(about = "Show Runs")]
    Show(ShowRunsOptions),
}

#[derive(Parser, Debug)]
pub struct ShowRunsOptions {
    #[clap(short = 'm', long, help = "Show machine validation runs of a machine")]
    pub machine: Option<MachineId>,

    #[clap(long, default_value = "false", help = "run history")]
    pub history: bool,
}
