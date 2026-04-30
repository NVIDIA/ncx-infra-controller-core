/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

/// Delete a compute allocation.
pub async fn delete(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let id = args.id;
    api_client.0.delete_compute_allocation(args).await?;
    println!("Deleted compute allocation {} successfully.", id);
    Ok(())
}
