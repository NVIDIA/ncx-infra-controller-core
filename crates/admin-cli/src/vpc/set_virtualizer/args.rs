/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_network::virtualization::VpcVirtualizationType;
use carbide_uuid::vpc::VpcId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The VPC ID for the VPC to update")]
    pub id: VpcId,
    #[clap(help = "The virtualizer to use for this VPC")]
    pub virtualizer: VpcVirtualizationType,
}
