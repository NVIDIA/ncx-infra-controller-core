/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::network::NetworkSegmentId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        default_value(None),
        help = "The network segment to query, leave empty for all (default)"
    )]
    pub network: Option<NetworkSegmentId>,

    #[clap(short, long, help = "The Tenant Org ID to query")]
    pub tenant_org_id: Option<String>,

    #[clap(short, long, help = "The VPC name to query")]
    pub name: Option<String>,
}
