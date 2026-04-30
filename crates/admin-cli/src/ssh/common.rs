/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct BmcCredentials {
    #[clap(help = "BMC IP Address")]
    pub bmc_ip_address: SocketAddr,
    #[clap(help = "BMC Username")]
    pub bmc_username: String,
    #[clap(help = "BMC Password")]
    pub bmc_password: String,
}

#[derive(Parser, Debug, Clone)]
pub struct SshArgs {
    #[clap(flatten)]
    pub credentials: BmcCredentials,
}
