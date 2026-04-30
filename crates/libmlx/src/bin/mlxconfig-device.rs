/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use libmlx::device::cmd::{Cli, dispatch_command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    dispatch_command(cli)
}
