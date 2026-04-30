/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct ShowSkuOptions {
    #[clap(help = "Show SKU details")]
    pub sku_id: Option<String>,
}

#[derive(Parser, Debug)]
pub struct CreateSkuOptions {
    #[clap(help = "The filename of the SKU data")]
    pub filename: String,
    #[clap(help = "override the ID of the SKU in the file data", long)]
    pub id: Option<String>,
}
