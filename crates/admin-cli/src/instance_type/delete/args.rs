/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge::DeleteInstanceTypeRequest;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "Instance type ID to delete")]
    pub id: String,
}

impl From<Args> for DeleteInstanceTypeRequest {
    fn from(args: Args) -> Self {
        DeleteInstanceTypeRequest { id: args.id }
    }
}
