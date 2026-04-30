/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::admin_cli::output::OutputFormat;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete(
    args: &Args,
    _output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    api_client.0.delete_vpc_peering(args.id).await?;
    println!("Deleted VPC peering {} successfully", args.id);
    Ok(())
}
