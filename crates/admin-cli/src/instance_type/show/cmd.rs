/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use ::rpc::forge::FindInstanceTypesByIdsRequest;

use super::args::Args;
use crate::instance_type::common::convert_itypes_to_table;
use crate::rpc::ApiClient;

/// Show one or more InstanceTypes.
/// If only a single InstanceType is found, verbose output is used
/// automatically.
pub async fn show(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
    page_size: usize,
    verbose: bool,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let itypes = if let Some(id) = args.id {
        vec![
            api_client
                .0
                .find_instance_types_by_ids(FindInstanceTypesByIdsRequest {
                    // Admin CLI doesn't need to filter on tenant org.
                    // When the time comes, the API will automatically
                    // filter if the caller is not a provider.
                    tenant_organization_id: None,
                    include_allocation_stats: args.show_stats.unwrap_or_default(),
                    instance_type_ids: vec![id],
                })
                .await?
                .instance_types
                .pop()
                .ok_or(CarbideCliError::Empty)?,
        ]
    } else {
        api_client.get_all_instance_types(page_size).await?
    };

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&itypes).map_err(CarbideCliError::JsonError)?
        );
    } else if itypes.len() == 1 {
        convert_itypes_to_table(&itypes, true)?.printstd();
    } else {
        convert_itypes_to_table(&itypes, verbose)?.printstd();
    }

    Ok(())
}
