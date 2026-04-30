/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        help = "Rack ID or name to delete (should not have any associated compute trays, nvlink switches or power shelves)"
    )]
    pub identifier: String,
}

impl From<Args> for ::rpc::forge::DeleteRackRequest {
    fn from(args: Args) -> Self {
        Self {
            id: args.identifier,
        }
    }
}
