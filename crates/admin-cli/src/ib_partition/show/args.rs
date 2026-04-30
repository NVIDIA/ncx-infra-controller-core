/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::infiniband::IBPartitionId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        default_value(None),
        help = "The InfiniBand Partition ID to query, leave empty for all (default)"
    )]
    pub id: Option<IBPartitionId>,

    #[clap(short, long, help = "The Tenant Org ID to query")]
    pub tenant_org_id: Option<String>,

    #[clap(short, long, help = "The InfiniBand Partition name to query")]
    pub name: Option<String>,
}
