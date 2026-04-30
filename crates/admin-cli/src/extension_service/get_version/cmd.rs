/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn handle_get_version(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let versions = api_client
        .0
        .get_dpu_extension_service_versions_info(args)
        .await?;

    println!("{}", serde_json::to_string_pretty(&versions.version_infos)?);

    Ok(())
}
