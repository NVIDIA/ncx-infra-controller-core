/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::output::OutputFormat;
use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};

use super::args::Args;
use crate::rpc::ApiClient;
use crate::vpc_peering::convert_vpc_peerings_to_table;

pub async fn create(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let vpc_peering = api_client.0.create_vpc_peering(args).await?;

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&vpc_peering).map_err(CarbideCliError::JsonError)?
        );
    } else {
        convert_vpc_peerings_to_table(&[vpc_peering])?.printstd();
    }

    Ok(())
}
