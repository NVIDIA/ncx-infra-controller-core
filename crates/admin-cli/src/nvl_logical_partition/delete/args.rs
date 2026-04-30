/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;
use ::rpc::forge as forgerpc;
use carbide_uuid::nvlink::NvLinkLogicalPartitionId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'n', long, help = "name of the partition")]
    pub name: String,
}

impl TryFrom<Args> for forgerpc::NvLinkLogicalPartitionDeletionRequest {
    type Error = CarbideCliError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let uuid: NvLinkLogicalPartitionId = uuid::Uuid::parse_str(&args.name)
            .map_err(|_| CarbideCliError::GenericError("UUID Conversion failed.".to_string()))?
            .into();
        Ok(Self { id: Some(uuid) })
    }
}
