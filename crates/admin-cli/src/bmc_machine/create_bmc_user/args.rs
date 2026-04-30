/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use mac_address::MacAddress;
use rpc::forge as forgerpc;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, short, help = "IP of the BMC where we want to create a new user")]
    pub ip_address: Option<String>,
    #[clap(long, help = "MAC of the BMC where we want to create a new user")]
    pub mac_address: Option<MacAddress>,
    #[clap(
        long,
        short,
        help = "ID of the machine where we want to create a new user"
    )]
    pub machine: Option<String>,

    #[clap(long, short, help = "Username of new BMC account")]
    pub username: String,
    #[clap(long, short, help = "Password of new BMC account")]
    pub password: String,
    #[clap(
        long,
        short,
        help = "Role of new BMC account ('administrator', 'operator', 'readonly', 'noaccess')"
    )]
    pub role_id: Option<String>,
}

impl From<Args> for forgerpc::CreateBmcUserRequest {
    fn from(args: Args) -> Self {
        let bmc_endpoint_request = if args.ip_address.is_some() || args.mac_address.is_some() {
            Some(forgerpc::BmcEndpointRequest {
                ip_address: args.ip_address.unwrap_or_default(),
                mac_address: args.mac_address.map(|mac| mac.to_string()),
            })
        } else {
            None
        };

        Self {
            bmc_endpoint_request,
            machine_id: args.machine,
            create_username: args.username,
            create_password: args.password,
            create_role_id: args.role_id,
        }
    }
}
