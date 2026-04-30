/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn set_primary_dpu(api_client: &ApiClient, args: Args) -> CarbideCliResult<()> {
    api_client.0.set_primary_dpu(args).await?;
    Ok(())
}
