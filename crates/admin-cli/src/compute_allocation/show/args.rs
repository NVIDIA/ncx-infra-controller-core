/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge::FindComputeAllocationIdsRequest;
use carbide_uuid::compute_allocation::ComputeAllocationId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(
        short = 'i',
        long,
        help = "Optional, compute allocation ID to restrict the search"
    )]
    pub id: Option<ComputeAllocationId>,

    #[clap(
        short = 't',
        long,
        help = "Optional, tenant organization ID used to filter results"
    )]
    pub tenant_organization_id: Option<String>,

    #[clap(short = 'n', long, help = "Optional, name used to filter results")]
    pub name: Option<String>,

    #[clap(long, help = "Optional, instance type ID used to filter results")]
    pub instance_type_id: Option<String>,
}

impl From<Args> for FindComputeAllocationIdsRequest {
    fn from(args: Args) -> Self {
        FindComputeAllocationIdsRequest {
            name: args.name,
            tenant_organization_id: args.tenant_organization_id,
            instance_type_id: args.instance_type_id,
        }
    }
}
