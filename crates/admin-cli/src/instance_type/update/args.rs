/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "Instance type ID to update")]
    pub id: String,

    #[clap(short = 'n', long, help = "Name of the instance type")]
    pub name: Option<String>,

    #[clap(short = 'd', long, help = "Description of the instance type")]
    pub description: Option<String>,

    #[clap(
        short = 'l',
        long,
        help = "JSON map of simple key:value pairs to be applied as labels to the instance type - will COMPLETELY overwrite any existing labels"
    )]
    pub labels: Option<String>,

    #[clap(
        short = 'f',
        long,
        help = "Optional, JSON array containing a set of instance type capability filters - will COMPLETELY overwrite any existing filters"
    )]
    pub desired_capabilities: Option<String>,

    #[clap(
        short = 'v',
        long,
        help = "Optional, version to use for comparison when performing the update, which will be rejected if the actual version of the record does not match the value of this parameter"
    )]
    pub version: Option<String>,
}
