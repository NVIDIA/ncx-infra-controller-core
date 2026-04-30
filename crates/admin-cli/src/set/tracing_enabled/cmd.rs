/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::ConfigSetting;

use crate::rpc::ApiClient;

pub async fn tracing_enabled(value: bool, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client
        .set_dynamic_config(ConfigSetting::TracingEnabled, value.to_string(), None)
        .await
}
