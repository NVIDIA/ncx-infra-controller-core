/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineInterfaceId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        short,
        long,
        action,
        conflicts_with = "interface_id",
        help = "Show all machine interfaces (DEPRECATED)"
    )]
    pub all: bool,

    #[clap(
        default_value(None),
        help = "The interface ID to query, leave empty for all (default)"
    )]
    pub interface_id: Option<MachineInterfaceId>,

    #[clap(long, action)]
    pub more: bool,
}
