/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::ConfigSetting;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn log_filter(opts: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client
        .set_dynamic_config(ConfigSetting::LogFilter, opts.filter, Some(opts.expiry))
        .await
}
