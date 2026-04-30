/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};

use super::args::Args;
use crate::network_security_group::common::convert_nsgs_to_table;
use crate::rpc::ApiClient;

/// Show one or more NSGs.
/// If only a single NSG is found, verbose output is used
/// automatically.
pub async fn show(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
    page_size: usize,
    verbose: bool,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let mut nsgs = Vec::new();
    if let Some(id) = args.id {
        let nsg = api_client.get_single_network_security_group(id).await?;
        nsgs.push(nsg);
    } else {
        nsgs = api_client
            .get_all_network_security_groups(page_size)
            .await?;
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&nsgs).map_err(CarbideCliError::JsonError)?
        );
    } else if nsgs.len() == 1 {
        convert_nsgs_to_table(&nsgs, true)?.printstd();
    } else {
        convert_nsgs_to_table(&nsgs, verbose)?.printstd();
    }

    Ok(())
}
