/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

/// Delete a network security group.
pub async fn delete(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let id = args.id.clone();
    api_client.0.delete_network_security_group(args).await?;
    println!("Deleted network security group {} successfully.", id);
    Ok(())
}
