/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Firmware configuration ID to set as default.")]
    pub firmware_id: String,
}

impl From<Args> for rpc::forge::RackFirmwareSetDefaultRequest {
    fn from(args: Args) -> Self {
        Self {
            firmware_id: args.firmware_id,
        }
    }
}
