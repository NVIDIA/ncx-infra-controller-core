/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, OutputFormat};
use prettytable::{Table, row};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn create(
    opts: Args,
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let request: rpc::forge::RackFirmwareCreateRequest = opts.try_into()?;
    let result = api_client.0.create_rack_firmware(request).await?;

    if format == OutputFormat::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        let mut table = Table::new();
        table.add_row(row!["ID", result.id]);
        let hw_type = result
            .rack_hardware_type
            .as_ref()
            .map(|t| t.value.as_str())
            .unwrap_or("N/A");
        table.add_row(row!["Hardware Type", hw_type]);
        table.add_row(row!["Default", result.is_default]);
        table.add_row(row!["Available", result.available]);
        table.add_row(row!["Created", result.created]);
        table.printstd();
    }

    Ok(())
}
