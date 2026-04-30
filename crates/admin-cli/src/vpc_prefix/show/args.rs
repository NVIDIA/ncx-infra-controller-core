/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::vpc::VpcId;
use clap::Parser;
use ipnet::IpNet;

use crate::vpc_prefix::common::VpcPrefixSelector;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        name = "VpcPrefixSelector",
        help = "The VPC prefix (by ID or exact unique prefix) to show (omit for all)"
    )]
    pub prefix_selector: Option<VpcPrefixSelector>,

    #[clap(
        long,
        name = "vpc-id",
        value_name = "VpcId",
        help = "Search by VPC ID",
        conflicts_with = "VpcPrefixSelector"
    )]
    pub vpc_id: Option<VpcId>,

    #[clap(
        long,
        name = "contains",
        value_name = "address-or-prefix",
        help = "Search by an address or prefix the VPC prefix contains",
        conflicts_with_all = ["VpcPrefixSelector", "contained-by"],
    )]
    pub contains: Option<IpNet>,

    #[clap(
        long,
        name = "contained-by",
        value_name = "prefix",
        help = "Search by a prefix containing the VPC prefix",
        conflicts_with_all = ["VpcPrefixSelector", "contains"],
    )]
    pub contained_by: Option<IpNet>,
}
