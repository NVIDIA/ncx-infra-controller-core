/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::vpc_peering::VpcPeeringId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, required(true), help = "The ID of the VPC peering to delete")]
    pub id: VpcPeeringId,
}
