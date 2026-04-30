/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "Optional, network security group ID to restrict the search")]
    pub id: Option<String>,
}
