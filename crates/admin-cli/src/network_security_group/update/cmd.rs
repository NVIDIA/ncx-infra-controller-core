/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};

use super::args::Args;
use crate::network_security_group::common::convert_nsgs_to_table;
use crate::rpc::ApiClient;

/// Update a network security group.
/// On successful update, the details of the
/// group will be displayed.
pub async fn update(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let id = args.id;

    let nsg = api_client
        .get_single_network_security_group(id.clone())
        .await?;

    let mut metadata = nsg.metadata.unwrap_or_default();
    let (mut rules, mut stateful_egress) = {
        let nsg = nsg.attributes.unwrap_or_default();
        (nsg.rules, nsg.stateful_egress)
    };

    if let Some(d) = args.description {
        metadata.description = d;
    }

    if let Some(n) = args.name {
        metadata.name = n;
    }

    if let Some(l) = args.labels {
        metadata.labels = serde_json::from_str(&l)?;
    }

    if let Some(r) = args.rules {
        rules = serde_json::from_str(&r)?;
    }

    if let Some(s) = args.stateful_egress {
        stateful_egress = s;
    }

    let nsg = api_client
        .update_network_security_group(
            id,
            args.tenant_organization_id,
            metadata,
            args.version,
            stateful_egress,
            rules,
        )
        .await?;

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
