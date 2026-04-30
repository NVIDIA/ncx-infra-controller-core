/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::RackId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Rack ID of expected rack to delete.")]
    pub rack_id: RackId,
}

impl From<Args> for rpc::forge::ExpectedRackRequest {
    fn from(args: Args) -> Self {
        rpc::forge::ExpectedRackRequest {
            rack_id: args.rack_id.to_string(),
        }
    }
}
