/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use super::super::common::SshArgs;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(flatten)]
    pub ssh_args: SshArgs,
    #[clap(help = "BFB Path")]
    pub bfb_path: String,
}
