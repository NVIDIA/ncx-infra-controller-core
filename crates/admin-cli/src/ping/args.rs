/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(
        short,
        long,
        default_value("1.0"),
        help = "Wait interval seconds between sending each request. Real number allowed with dot as a decimal separator."
    )]
    pub interval: f32,
}
