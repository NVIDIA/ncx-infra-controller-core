/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::network::NetworkSegmentId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, help = "Id of the network segment")]
    pub id: NetworkSegmentId,
}

impl From<Args> for ::rpc::forge::NetworkSegmentDeletionRequest {
    fn from(args: Args) -> Self {
        Self { id: Some(args.id) }
    }
}
