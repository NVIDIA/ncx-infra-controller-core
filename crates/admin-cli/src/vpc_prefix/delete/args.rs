/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::vpc::VpcPrefixId;
use clap::Parser;
use rpc::forge::VpcPrefixDeletionRequest;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(value_name = "VpcPrefixId")]
    pub vpc_prefix_id: VpcPrefixId,
}

impl From<Args> for VpcPrefixDeletionRequest {
    fn from(args: Args) -> Self {
        VpcPrefixDeletionRequest {
            id: Some(args.vpc_prefix_id),
        }
    }
}
