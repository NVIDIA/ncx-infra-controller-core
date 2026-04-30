/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use rpc::forge::SkuIdList;

use crate::rpc::ApiClient;

pub async fn delete(sku_id: String, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client
        .0
        .delete_sku(SkuIdList { ids: vec![sku_id] })
        .await?;
    Ok(())
}
