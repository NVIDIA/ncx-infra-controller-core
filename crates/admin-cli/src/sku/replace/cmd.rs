/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliResult, OutputFormat};

use super::super::common::CreateSkuOptions;
use crate::rpc::ApiClient;
use crate::sku::show::cmd::show_skus_table;

pub async fn replace(
    args: CreateSkuOptions,
    api_client: &ApiClient,
    output: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
    output_format: &OutputFormat,
) -> CarbideCliResult<()> {
    let file_data = std::fs::read_to_string(args.filename)?;
    let mut sku: rpc::forge::Sku = serde_json::de::from_str(&file_data)?;
    sku.id = args.id.unwrap_or(sku.id);

    let updated_sku = api_client.0.replace_sku(sku).await?;
    show_skus_table(output, output_format, vec![updated_sku]).await?;
    Ok(())
}
