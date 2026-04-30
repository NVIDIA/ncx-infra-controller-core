/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use clap::builder::BoolishValueParser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(num_args = 1, value_parser = BoolishValueParser::new(), action = clap::ArgAction::Set, value_name = "true|false")]
    pub value: bool,
}
