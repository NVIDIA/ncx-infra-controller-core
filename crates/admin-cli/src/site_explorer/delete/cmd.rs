/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete_endpoint(api_client: &ApiClient, opts: Args) -> CarbideCliResult<()> {
    let response = api_client.0.delete_explored_endpoint(opts.address).await?;

    if response.deleted {
        println!(
            "{}",
            response
                .message
                .unwrap_or_else(|| "Endpoint deleted successfully.".to_string())
        );
    } else {
        eprintln!(
            "{}",
            response
                .message
                .unwrap_or_else(|| "Failed to delete endpoint.".to_string())
        );
    }
    Ok(())
}
