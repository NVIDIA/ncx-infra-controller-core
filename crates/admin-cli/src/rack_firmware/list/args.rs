/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, help = "Show only available configurations.")]
    pub only_available: bool,
    #[clap(help = "Filter by rack hardware type.")]
    pub rack_hardware_type: Option<String>,
}

impl From<Args> for rpc::forge::RackFirmwareSearchFilter {
    fn from(args: Args) -> Self {
        Self {
            only_available: args.only_available,
            rack_hardware_type: args
                .rack_hardware_type
                .map(|v| rpc::common::RackHardwareType { value: v }),
        }
    }
}
