/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(
        help = "Path to devenv config file. Usually this is in forged repo at envs/local-dev/site/site-controller/files/generated/devenv_config.toml"
    )]
    pub path: String,

    #[clap(long, short, help = "Vpc prefix or network segment?")]
    pub mode: NetworkChoice,
}

#[derive(ValueEnum, Parser, Debug, Clone, PartialEq)]
pub enum NetworkChoice {
    NetworkSegment,
    VpcPrefix,
}
