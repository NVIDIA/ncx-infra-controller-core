/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[clap(group(
        ArgGroup::new("grow")
        .required(true)
        .args(&["filename"])))]
pub struct Args {
    #[clap(short, long)]
    pub filename: String,
}

impl TryFrom<Args> for ::rpc::forge::GrowResourcePoolRequest {
    type Error = std::io::Error;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let text = std::fs::read_to_string(&args.filename)?;
        Ok(Self { text })
    }
}
