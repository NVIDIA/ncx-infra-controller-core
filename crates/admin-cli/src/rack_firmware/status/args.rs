/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Job ID to check status for (from apply output)")]
    pub job_id: String,
}

impl From<Args> for rpc::forge::RackFirmwareJobStatusRequest {
    fn from(args: Args) -> Self {
        Self {
            job_id: args.job_id,
        }
    }
}
