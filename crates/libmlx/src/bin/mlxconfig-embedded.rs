/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use libmlx::embedded::cmd::{Cli, LogLevel, run_cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // While --log-level is used to control the log level here, you
    // can also set RUST_LOG in your environment to override.
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level = match cli.log_level {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };
        tracing_subscriber::EnvFilter::new(level)
    });

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    if let Err(e) = run_cli(cli).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
