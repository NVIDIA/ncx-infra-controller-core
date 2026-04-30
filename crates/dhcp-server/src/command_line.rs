/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[clap(name = "forge-dhcp-server")]
#[clap(author = "Slack channel #swngc-forge-dev")]
pub struct Args {
    #[arg(long, help = "Interface name where to bind this server.")]
    pub interfaces: Vec<String>,

    #[arg(
        long,
        help = "DHCP Config file path.",
        default_value = "/var/support/forge-dhcp/conf/dhcp.yaml"
    )]
    pub dhcp_config: String,

    #[arg(
        long,
        help = "DPU Agent provided input file path for IP selection. Defaults to \
                /var/support/forge-dhcp/conf/host.yaml when --grpc-listen-addr is set."
    )]
    pub host_config: Option<String>,

    #[arg(short, long, value_enum, default_value_t=ServerMode::Dpu)]
    pub mode: ServerMode,

    #[arg(
        long,
        help = "gRPC server listen address for config hot-reload (e.g. 0.0.0.0:50051). \
                When omitted the gRPC server is not started and config reload is disabled."
    )]
    pub grpc_listen_addr: Option<String>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ServerMode {
    Dpu,
    Controller,
}

impl Args {
    pub fn load() -> Self {
        Self::parse()
    }
}
