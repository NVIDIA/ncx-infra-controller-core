/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(
        short = 'i',
        long,
        help = "Optional, instance type ID to restrict the search"
    )]
    pub id: Option<String>,

    #[clap(
        short = 's',
        long,
        help = "Optional, show counts for allocations of instance types"
    )]
    pub show_stats: Option<bool>,
}
