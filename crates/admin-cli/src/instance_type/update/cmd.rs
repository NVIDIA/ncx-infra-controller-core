/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use ::rpc::forge::{
    FindInstanceTypesByIdsRequest, InstanceTypeAttributes, UpdateInstanceTypeRequest,
};

use super::args::Args;
use crate::instance_type::common::convert_itypes_to_table;
use crate::rpc::ApiClient;

/// Update an instance type.
/// On successful update, the details of the
/// type will be displayed.
pub async fn update(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let id = args.id;

    let itype = api_client
        .0
        .find_instance_types_by_ids(FindInstanceTypesByIdsRequest {
            tenant_organization_id: None,
            include_allocation_stats: false,
            instance_type_ids: vec![id.clone()],
        })
        .await?
        .instance_types
        .pop()
        .ok_or(CarbideCliError::Empty)?;

    let mut metadata = itype.metadata.unwrap_or_default();

    if let Some(d) = args.description {
        metadata.description = d;
    }

    if let Some(n) = args.name {
        metadata.name = n;
    }

    if let Some(l) = args.labels {
        metadata.labels = serde_json::from_str(&l)?;
    }

    let instance_type_attributes = args
        .desired_capabilities
        .map(|d| {
            serde_json::from_str(&d).map(|desired_capabilities| InstanceTypeAttributes {
                desired_capabilities,
            })
        })
        .transpose()?;

    let itype = api_client
        .0
        .update_instance_type(UpdateInstanceTypeRequest {
            id,
            metadata: Some(metadata),
            if_version_match: args.version,
            instance_type_attributes,
        })
        .await?
        .instance_type
        .ok_or(CarbideCliError::Empty)?;

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&itype).map_err(CarbideCliError::JsonError)?
        );
    } else {
        convert_itypes_to_table(&[itype], true)?.printstd();
    }

    Ok(())
}
