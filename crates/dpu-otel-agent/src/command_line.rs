/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(name = "forge-dpu-otel-agent")]
pub struct Options {
    /// The path to the agent configuration file overrides.
    /// This file will hold data in the `AgentConfig` format.
    #[clap(long)]
    pub config_path: Option<PathBuf>,

    #[clap(subcommand)]
    pub cmd: Option<AgentCommand>,
}

#[derive(Parser, Debug)]
pub enum AgentCommand {
    #[clap(about = "Run is the normal command. Runs main loop forever.")]
    Run(Box<RunOptions>),
}

#[derive(Parser, Debug)]
pub struct RunOptions {}

impl Options {
    pub fn load() -> Self {
        Self::parse()
    }
}
