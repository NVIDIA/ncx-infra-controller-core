/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

/// Delete an instance type.
pub async fn delete(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let id = args.id.clone();
    api_client.0.delete_instance_type(args).await?;
    println!("Deleted instance type {} successfully.", id);
    Ok(())
}
