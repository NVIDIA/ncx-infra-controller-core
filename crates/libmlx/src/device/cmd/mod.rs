/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::{Parser, Subcommand};

use crate::device::cmd::device::args::DeviceArgs;
pub mod device;

// Cli represents the main CLI structure for the application.
#[derive(Parser)]
#[command(
    author,
    version,
    about = "mlxconfig-device - mellanox device discovery"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

// Commands defines the available top-level commands.
#[derive(Subcommand)]
pub enum Commands {
    // Device management commands for discovering and
    // inspecting Mellanox devices.
    Device(DeviceArgs),
}

// dispatch_command routes CLI commands to their
// appropriate handlers.
pub fn dispatch_command(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Device(args) => crate::device::cmd::device::cmds::handle(args),
    }
}
