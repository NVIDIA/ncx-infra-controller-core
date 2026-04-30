/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("group").required(true).multiple(true).args(&["description", "device_type"])))]
pub struct Args {
    #[clap(help = "SKU ID of the SKU to update")]
    pub sku_id: String,
    #[clap(help = "Update the SKU's description", long, group("group"))]
    pub description: Option<String>,
    #[clap(help = "Update the SKU's device type", long, group("group"))]
    pub device_type: Option<String>,
}

impl From<Args> for ::rpc::forge::SkuUpdateMetadataRequest {
    fn from(value: Args) -> Self {
        ::rpc::forge::SkuUpdateMetadataRequest {
            sku_id: value.sku_id,
            description: value.description,
            device_type: value.device_type,
        }
    }
}
