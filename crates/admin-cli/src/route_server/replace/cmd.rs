/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use crate::route_server::common::AddressArgs;
use crate::rpc::ApiClient;

pub async fn replace(args: AddressArgs, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client.0.replace_route_servers(args).await?;

    Ok(())
}
