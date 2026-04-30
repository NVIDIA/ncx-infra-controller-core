/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge::RouteServerSourceType;

// AddressArgs is used for add/remove/replace operations
// for route server addresses, with support for overriding
// the source_type to not be admin_api, and make ephemeral
// changes against whatever was loaded up via the config
// file at start.
#[derive(Parser, Debug)]
pub struct AddressArgs {
    #[arg(value_delimiter = ',', help = "Comma-separated list of IP addresses")]
    pub ip: Vec<std::net::IpAddr>,

    // The optional source_type to set. If unset, this
    // defaults to admin_api, which is what we'd expect.
    // Override with --source_type=config to make
    // ephemeral changes to config file-based entries,
    // which is really intended for break-glass types
    // of scenarios.
    #[arg(
        long,
        default_value = "admin_api",
        help = "The source_type to use for the target addresses. Defaults to admin_api."
    )]
    pub source_type: RouteServerSourceType,
}

impl From<AddressArgs> for ::rpc::forge::RouteServers {
    fn from(args: AddressArgs) -> Self {
        Self {
            route_servers: args.ip.iter().map(ToString::to_string).collect(),
            source_type: args.source_type as i32,
        }
    }
}
