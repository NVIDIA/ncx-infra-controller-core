/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use ::rpc::forge::SkuList;

use super::args::Args;
use crate::rpc::ApiClient;
use crate::sku::show::cmd::show_skus_table;

pub async fn create(
    args: Args,
    api_client: &ApiClient,
    output: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
    output_format: &OutputFormat,
) -> CarbideCliResult<()> {
    let file_data = std::fs::read_to_string(args.filename)?;
    // attempt to deserialize a single sku.  if it fails try to deserialize as a SkuList
    let mut sku_list = match serde_json::de::from_str(&file_data) {
        Ok(sku) => SkuList { skus: vec![sku] },
        Err(e) => serde_json::de::from_str(&file_data).map_err(|_| e)?,
    };
    if let Some(id) = args.id {
        if sku_list.skus.len() != 1 {
            return Err(CarbideCliError::GenericError(
                "ID cannot be specified when creating multiple SKUs".to_string(),
            ));
        }
        sku_list.skus[0].id = id;
    }
    let sku_ids = api_client.0.create_sku(sku_list).await?;
    let sku_list = api_client.0.find_skus_by_ids(sku_ids.ids).await?;
    show_skus_table(output, output_format, sku_list.skus).await?;
    Ok(())
}
