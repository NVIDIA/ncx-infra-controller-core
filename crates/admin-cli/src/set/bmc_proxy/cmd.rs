/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::ConfigSetting;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn bmc_proxy(opts: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    if opts.enabled {
        api_client
            .set_dynamic_config(
                ConfigSetting::BmcProxy,
                opts.proxy.unwrap_or_default(),
                None,
            )
            .await
    } else {
        api_client
            .set_dynamic_config(ConfigSetting::BmcProxy, String::new(), None)
            .await
    }
}
