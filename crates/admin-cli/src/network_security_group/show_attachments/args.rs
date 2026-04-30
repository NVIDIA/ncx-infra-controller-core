/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "network security group ID to query")]
    pub id: String,

    #[clap(
        short = 'a',
        long,
        help = "include indirect relationships (objects that are inheriting the NSG from a parent object)"
    )]
    pub include_indirect: bool,
}
