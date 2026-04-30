/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short = 'i', long, help = "The extension service ID")]
    pub service_id: String,

    #[clap(short = 'v', long, help = "Version string to filter by (optional)")]
    pub version: Option<String>,
}

impl From<Args> for ::rpc::forge::FindInstancesByDpuExtensionServiceRequest {
    fn from(args: Args) -> Self {
        Self {
            service_id: args.service_id,
            version: args.version,
        }
    }
}
