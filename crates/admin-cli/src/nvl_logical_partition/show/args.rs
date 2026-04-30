/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        default_value(""),
        help = "Optional, Logical Partition ID to search for"
    )]
    pub id: String,
    #[clap(short, long, help = "Optional, Logical Partition Name to search for")]
    pub name: Option<String>,
}
