/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};

use super::args::Args;
use crate::network_security_group::common::convert_nsgs_to_table;
use crate::rpc::ApiClient;

/// Create a network security group.
/// On successful creation, the details of the
/// new group will be displayed.
pub async fn create(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let req: ::rpc::forge::CreateNetworkSecurityGroupRequest = args.try_into()?;
    let nsg = api_client
        .0
        .create_network_security_group(req)
        .await?
        .network_security_group
        .ok_or(CarbideCliError::Empty)?;

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&nsg).map_err(CarbideCliError::JsonError)?
        );
    } else {
        convert_nsgs_to_table(&[nsg], true)?.printstd();
    }

    Ok(())
}
