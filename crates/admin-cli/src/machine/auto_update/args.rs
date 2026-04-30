/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::{ArgGroup, Parser};

#[derive(Parser, Debug, Clone)]
#[clap(group(ArgGroup::new("autoupdate_action").required(true).args(&["enable", "disable", "clear"])))]
pub struct Args {
    #[clap(long, help = "Machine ID of the host to change")]
    pub machine: MachineId,
    #[clap(
        short = 'e',
        long,
        action,
        help = "Enable auto updates even if globally disabled or individually disabled by config files"
    )]
    pub enable: bool,
    #[clap(
        short = 'd',
        long,
        action,
        help = "Disable auto updates even if globally enabled or individually enabled by config files"
    )]
    pub disable: bool,
    #[clap(
        short = 'c',
        long,
        action,
        help = "Perform auto updates according to config files"
    )]
    pub clear: bool,
}
