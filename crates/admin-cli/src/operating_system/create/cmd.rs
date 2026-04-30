/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use ::rpc::forge::CreateOperatingSystemRequest;

use super::args::Args;
use crate::operating_system::common::{str_to_ipxe_template_id, str_to_os_id};
use crate::rpc::ApiClient;

pub async fn create(opts: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let id = opts.id.as_deref().map(str_to_os_id).transpose()?;
    let ipxe_template_id = opts
        .ipxe_template_id
        .as_deref()
        .map(str_to_ipxe_template_id)
        .transpose()?;

    let os = api_client
        .0
        .create_operating_system(CreateOperatingSystemRequest {
            name: opts.name,
            tenant_organization_id: opts.org,
            description: opts.description,
            is_active: opts.is_active.unwrap_or(true),
            allow_override: opts.allow_override,
            phone_home_enabled: opts.phone_home_enabled,
            user_data: opts.user_data,
            id,
            ipxe_script: opts.ipxe_script,
            ipxe_template_id,
            ipxe_template_parameters: opts.params,
            ipxe_template_artifacts: vec![],
        })
        .await
        .map_err(CarbideCliError::from)?;

    println!(
        "Operating system created: {} (id={})",
        os.name,
        os.id.map(|u| u.to_string()).as_deref().unwrap_or("")
    );
    Ok(())
}
