/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, help = "ID of the machine to enable/disable lockdown")]
    pub machine: MachineId,
    #[clap(short, long, help = "Issue reboot to apply lockdown change")]
    pub reboot: bool,
    #[clap(
        long,
        conflicts_with = "disable",
        required_unless_present = "disable",
        help = "Enable lockdown"
    )]
    pub enable: bool,
    #[clap(
        long,
        conflicts_with = "enable",
        required_unless_present = "enable",
        help = "Disable lockdown"
    )]
    pub disable: bool,
}
