/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::vpc::VpcId;
use carbide_uuid::vpc_peering::VpcPeeringId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        long,
        conflicts_with = "vpc_id",
        help = "The ID of the VPC peering to show"
    )]
    pub id: Option<VpcPeeringId>,

    #[clap(
        long,
        conflicts_with = "id",
        help = "The ID of the VPC to show VPC peerings for"
    )]
    pub vpc_id: Option<VpcId>,
}
