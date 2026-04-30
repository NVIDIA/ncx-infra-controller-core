/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn handle_delete(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let req: ::rpc::forge::NvLinkLogicalPartitionDeletionRequest = args.try_into()?;
    api_client.0.delete_nv_link_logical_partition(req).await?;
    Ok(())
}
