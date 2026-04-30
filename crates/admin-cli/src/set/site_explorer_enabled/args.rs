/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(group = clap::ArgGroup::new("toggle").required(true))]
pub struct Args {
    #[clap(long, group = "toggle", help = "Enable site-explorer")]
    pub enable: bool,

    #[clap(long, group = "toggle", help = "Disable site-explorer")]
    pub disable: bool,
}

impl Args {
    pub fn is_enabled(&self) -> bool {
        self.enable
    }
}
