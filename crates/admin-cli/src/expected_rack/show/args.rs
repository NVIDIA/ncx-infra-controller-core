/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::RackId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        default_value(None),
        help = "Rack ID of the expected rack to show. Leave unset for all."
    )]
    pub rack_id: Option<RackId>,
}

impl From<&Args> for Option<rpc::forge::ExpectedRackRequest> {
    fn from(args: &Args) -> Self {
        args.rack_id
            .as_ref()
            .map(|id| rpc::forge::ExpectedRackRequest {
                rack_id: id.to_string(),
            })
    }
}
