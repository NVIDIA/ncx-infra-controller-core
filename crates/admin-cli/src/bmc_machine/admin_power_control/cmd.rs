/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn admin_power_control(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client.0.admin_power_control(args).await?;
    Ok(())
}
