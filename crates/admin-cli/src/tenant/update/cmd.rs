/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use rpc::forge::{FindTenantRequest, UpdateTenantRequest};

use super::args::Args;
use crate::rpc::ApiClient;
use crate::tenant::show::cmd::convert_tenants_to_table;

/// Update a tenant.
/// On successful update, the details of the
/// tenant will be displayed.
pub async fn update(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let id = args.tenant_org;

    let tenant = api_client
        .0
        .find_tenant(FindTenantRequest {
            tenant_organization_id: id.clone(),
        })
        .await?
        .tenant
        .ok_or(CarbideCliError::TenantNotFound(id.clone()))?;

    let mut metadata = tenant.metadata.unwrap_or_default();

    if let Some(n) = args.name {
        metadata.name = n;
    }

    let tenant = api_client
        .0
        .update_tenant(UpdateTenantRequest {
            organization_id: id.clone(),
            metadata: Some(metadata),
            if_version_match: args.version,
            routing_profile_type: args.routing_profile_type,
        })
        .await?
        .tenant
        .ok_or(CarbideCliError::TenantNotFound(id))?;

    match output_format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&tenant).map_err(CarbideCliError::JsonError)?
        ),
        OutputFormat::Yaml => println!(
            "{}",
            serde_yaml::to_string(&tenant).map_err(CarbideCliError::YamlError)?
        ),
        OutputFormat::Csv => {
            convert_tenants_to_table(&[tenant])?
                .to_csv(std::io::stdout())
                .map_err(CarbideCliError::CsvError)?
                .flush()?;
        }

        _ => convert_tenants_to_table(&[tenant])?.printstd(),
    }

    Ok(())
}
