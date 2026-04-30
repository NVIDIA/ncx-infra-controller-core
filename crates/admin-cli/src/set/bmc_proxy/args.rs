/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, action = clap::ArgAction::Set, help = "Enable site-explorer bmc_proxy")]
    pub enabled: bool,
    #[clap(long, action = clap::ArgAction::Set, help = "host:port string use as a proxy for talking to BMC's")]
    pub proxy: Option<String>,
}
