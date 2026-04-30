/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use crate::rpc::ApiClient;

pub async fn clear_error(api_client: &ApiClient, address: String) -> CarbideCliResult<()> {
    api_client.0.clear_site_exploration_error(address).await?;
    Ok(())
}
