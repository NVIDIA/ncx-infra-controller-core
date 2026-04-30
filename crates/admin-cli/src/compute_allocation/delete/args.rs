/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge::DeleteComputeAllocationRequest;
use carbide_uuid::compute_allocation::ComputeAllocationId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "Compute allocation ID to delete")]
    pub id: ComputeAllocationId,

    #[clap(
        short = 't',
        long,
        help = "Tenant organization ID for the compute allocation"
    )]
    pub tenant_organization_id: String,
}

impl From<Args> for DeleteComputeAllocationRequest {
    fn from(args: Args) -> Self {
        DeleteComputeAllocationRequest {
            id: Some(args.id),
            tenant_organization_id: args.tenant_organization_id,
        }
    }
}
