/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

/// ShowInstance is used for `cli instance show` configuration,
/// with the ability to filter by a combination of labels, tenant
/// org ID, and VPC ID.
//
// TODO: Possibly add the ability to filter by a list of tenant
// org IDs and/or VPC IDs.
#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        default_value(""),
        help = "The instance ID to query, leave empty for all (default)"
    )]
    pub id: String,

    #[clap(short, long, action)]
    pub extrainfo: bool,

    #[clap(short, long, help = "The Tenant Org ID to query")]
    pub tenant_org_id: Option<String>,

    #[clap(short, long, help = "The VPC ID to query.")]
    pub vpc_id: Option<String>,

    #[clap(long, help = "The key of label instance to query")]
    pub label_key: Option<String>,

    #[clap(long, help = "The value of label instance to query")]
    pub label_value: Option<String>,

    #[clap(long, help = "The instance type ID to query.")]
    pub instance_type_id: Option<String>,
}
