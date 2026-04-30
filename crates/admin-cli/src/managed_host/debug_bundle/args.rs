/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The host machine ID to collect logs for")]
    pub host_id: String,

    #[clap(
        long,
        help = "Start time: 'YYYY-MM-DD HH:MM:SS' or 'HH:MM:SS' (uses today's date). Default: local timezone, use --utc for UTC"
    )]
    pub start_time: String,

    #[clap(
        long,
        help = "End time: 'YYYY-MM-DD HH:MM:SS' or 'HH:MM:SS' (uses today's date). Defaults to current time if not provided. Default: local timezone, use --utc for UTC"
    )]
    pub end_time: Option<String>,

    #[clap(
        long,
        help = "Interpret start-time and end-time as UTC instead of local timezone"
    )]
    pub utc: bool,

    #[clap(
        long,
        default_value = "/tmp",
        help = "Output directory path for the debug bundle (default: /tmp)"
    )]
    pub output_path: String,

    #[clap(
        long,
        help = "Grafana base URL (e.g., https://grafana.example.com). If not provided, log collection is skipped."
    )]
    pub grafana_url: Option<String>,

    #[clap(
        long,
        default_value = "5000",
        help = "Batch size for log collection (default: 5000, max: 5000)"
    )]
    pub batch_size: u32,
}
