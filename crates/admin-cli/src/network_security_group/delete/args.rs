/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge::DeleteNetworkSecurityGroupRequest;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "Network security group ID to delete")]
    pub id: String,

    #[clap(
        short = 't',
        long,
        help = "Tenant organization ID of the network security group"
    )]
    pub tenant_organization_id: String,
}

impl From<Args> for DeleteNetworkSecurityGroupRequest {
    fn from(args: Args) -> Self {
        DeleteNetworkSecurityGroupRequest {
            id: args.id,
            tenant_organization_id: args.tenant_organization_id,
        }
    }
}
