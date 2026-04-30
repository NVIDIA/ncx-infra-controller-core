/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::RackId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Rack ID to apply firmware to")]
    pub rack_id: RackId,

    #[clap(help = "Firmware configuration ID to apply")]
    pub firmware_id: String,

    #[clap(help = "Firmware type: dev or prod", value_parser = ["dev", "prod"])]
    pub firmware_type: String,
}

impl From<Args> for rpc::forge::RackFirmwareApplyRequest {
    fn from(args: Args) -> Self {
        Self {
            rack_id: Some(args.rack_id),
            firmware_id: args.firmware_id,
            firmware_type: args.firmware_type,
        }
    }
}
