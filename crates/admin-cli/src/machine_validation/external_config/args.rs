/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub enum Args {
    #[clap(about = "Show External config")]
    Show(ExternalConfigShowOptions),

    #[clap(about = "Update External config")]
    AddUpdate(ExternalConfigAddOptions),

    #[clap(about = "Remove External config")]
    Remove(ExternalConfigRemoveOptions),
}

#[derive(Parser, Debug)]
pub struct ExternalConfigShowOptions {
    #[clap(short, long, help = "Machine validation external config names")]
    pub name: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ExternalConfigAddOptions {
    #[clap(short, long, help = "Name of the file to update")]
    pub file_name: String,
    #[clap(short, long, help = "Name of the config")]
    pub name: String,
    #[clap(short, long, help = "description of the file to update")]
    pub description: String,
}

#[derive(Parser, Debug)]
pub struct ExternalConfigRemoveOptions {
    #[clap(short, long, help = "Machine validation external config name")]
    pub name: String,
}
