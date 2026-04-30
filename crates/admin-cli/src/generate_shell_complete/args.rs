/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Cmd {
    #[clap(subcommand)]
    pub shell: Shell,
}

#[derive(Parser, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
pub enum Shell {
    Bash,
    Fish,
    Zsh,
}
