/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge as forgerpc;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'n', long, help = "name of the partition")]
    pub name: String,
    #[clap(short = 't', long, help = "tenant organization id of the partition")]
    pub tenant_organization_id: String,
}

impl From<Args> for forgerpc::NvLinkLogicalPartitionCreationRequest {
    fn from(args: Args) -> Self {
        let metadata = forgerpc::Metadata {
            name: args.name,
            labels: vec![forgerpc::Label {
                key: "cloud-unsafe-op".to_string(),
                value: Some("true".to_string()),
            }],
            ..Default::default()
        };
        Self {
            config: Some(forgerpc::NvLinkLogicalPartitionConfig {
                metadata: Some(metadata),
                tenant_organization_id: args.tenant_organization_id,
            }),
            id: None,
        }
    }
}
