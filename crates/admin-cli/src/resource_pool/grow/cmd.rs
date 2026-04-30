/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn grow(data: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let rpc_req: ::rpc::forge::GrowResourcePoolRequest = data.try_into()?;
    api_client.0.admin_grow_resource_pool(rpc_req).await?;
    Ok(())
}
