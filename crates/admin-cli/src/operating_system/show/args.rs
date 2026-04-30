/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "Operating system definition ID; omit to list all.")]
    pub id: Option<String>,

    #[clap(long, help = "Filter by organization identifier (when listing).")]
    pub org: Option<String>,
}
