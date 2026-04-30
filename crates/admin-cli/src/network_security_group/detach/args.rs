/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::instance::InstanceId;
use carbide_uuid::vpc::VpcId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(
        short = 'v',
        long,
        help = "Optional, VPC ID that should have the network security group removed"
    )]
    pub vpc_id: Option<VpcId>,

    #[clap(
        short = 'i',
        long,
        help = "Optional, Instance ID that should have the network security group removed"
    )]
    pub instance_id: Option<InstanceId>,
}
