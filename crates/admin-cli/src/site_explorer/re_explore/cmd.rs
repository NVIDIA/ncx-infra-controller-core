/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::ReExploreEndpointRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn re_explore(api_client: &ApiClient, opts: Args) -> CarbideCliResult<()> {
    api_client
        .0
        .re_explore_endpoint(ReExploreEndpointRequest {
            ip_address: opts.address,
            if_version_match: None,
        })
        .await?;
    Ok(())
}
