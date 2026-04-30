/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short = 'i', long = "id", help = "The extension service ID to delete")]
    pub service_id: String,

    #[clap(
        short = 'v',
        long,
        help = "Version strings to delete (optional, leave empty to keep all versions)",
        value_delimiter = ','
    )]
    pub versions: Vec<String>,
}

impl From<Args> for ::rpc::forge::DeleteDpuExtensionServiceRequest {
    fn from(args: Args) -> Self {
        Self {
            service_id: args.service_id,
            versions: args.versions,
        }
    }
}
