/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "ID of the configuration to retrieve")]
    pub id: String,
}

impl From<Args> for rpc::forge::RackFirmwareGetRequest {
    fn from(args: Args) -> Self {
        Self { id: args.id }
    }
}
