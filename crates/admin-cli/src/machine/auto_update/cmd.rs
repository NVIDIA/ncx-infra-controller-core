/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn autoupdate(cfg: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let _response = api_client.machine_set_auto_update(cfg).await?;
    Ok(())
}
