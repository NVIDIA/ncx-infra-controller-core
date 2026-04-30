/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use rpc::forge::RemoveSkuRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn unassign(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client
        .0
        .remove_sku_association(RemoveSkuRequest {
            machine_id: Some(args.machine_id),
            force: args.force,
        })
        .await?;
    Ok(())
}
