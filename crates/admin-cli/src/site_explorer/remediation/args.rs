/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "BMC IP address of the endpoint")]
    pub address: String,
    #[clap(long, help = "Pause remediation actions", conflicts_with = "resume")]
    pub pause: bool,
    #[clap(long, help = "Resume remediation actions", conflicts_with = "pause")]
    pub resume: bool,
}
