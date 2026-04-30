/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    /// The IP address we are looking to identify
    pub ip: std::net::IpAddr,
}

impl From<Args> for ::rpc::forge::FindIpAddressRequest {
    fn from(args: Args) -> Self {
        Self {
            ip: args.ip.to_string(),
        }
    }
}
