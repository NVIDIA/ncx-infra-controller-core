/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::vpc::VpcId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        default_value(None),
        help = "The VPC ID to query, leave empty for all (default)"
    )]
    pub id: Option<VpcId>,

    #[clap(short, long, help = "The Tenant Org ID to query")]
    pub tenant_org_id: Option<String>,

    #[clap(short, long, help = "The VPC name to query")]
    pub name: Option<String>,

    #[clap(long, help = "The key of VPC label to query")]
    pub label_key: Option<String>,

    #[clap(long, help = "The value of VPC label to query")]
    pub label_value: Option<String>,
}
