/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::vpc::VpcId;
use carbide_uuid::vpc_peering::VpcPeeringId;
use clap::Parser;
use rpc::forge::VpcPeeringCreationRequest;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The ID of one VPC ID to peer")]
    pub vpc1_id: VpcId,

    #[clap(help = "The ID of other VPC ID to peer")]
    pub vpc2_id: VpcId,

    #[clap(long, help = "Optional desired ID for the VPC peering")]
    pub id: Option<VpcPeeringId>,
}

impl From<Args> for VpcPeeringCreationRequest {
    fn from(args: Args) -> Self {
        VpcPeeringCreationRequest {
            vpc_id: Some(args.vpc1_id),
            peer_vpc_id: Some(args.vpc2_id),
            id: args.id,
        }
    }
}
