/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(short, long, action, help = "Display Runtime Config also.")]
    pub show_runtime_config: bool,
}
