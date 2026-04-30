/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, help = "Filter by firmware ID")]
    pub firmware_id: Option<String>,

    #[clap(long, help = "Filter by rack ID(s)")]
    pub rack_id: Vec<String>,
}

impl From<Args> for rpc::forge::RackFirmwareHistoryRequest {
    fn from(args: Args) -> Self {
        Self {
            firmware_id: args.firmware_id.unwrap_or_default(),
            rack_ids: args.rack_id,
        }
    }
}
