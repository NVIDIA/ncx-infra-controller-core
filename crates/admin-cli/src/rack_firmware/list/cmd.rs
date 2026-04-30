/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, OutputFormat};
use prettytable::{Cell, Row, Table};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn list(
    opts: Args,
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let result = api_client.0.list_rack_firmware(opts).await?;

    if format == OutputFormat::Json {
        println!("{}", serde_json::to_string_pretty(&result.configs)?);
    } else if result.configs.is_empty() {
        println!("No Rack firmware configurations found.");
    } else {
        let mut table = Table::new();
        table.set_titles(Row::new(vec![
            Cell::new("ID"),
            Cell::new("Hardware Type"),
            Cell::new("Default"),
            Cell::new("Available"),
            Cell::new("Created"),
            Cell::new("Updated"),
        ]));

        for config in result.configs {
            let hw_type = config
                .rack_hardware_type
                .as_ref()
                .map(|t| t.value.as_str())
                .unwrap_or("N/A");
            let default_marker = if config.is_default { "*" } else { "" };
            table.add_row(Row::new(vec![
                Cell::new(&config.id),
                Cell::new(hw_type),
                Cell::new(default_marker),
                Cell::new(&config.available.to_string()),
                Cell::new(&config.created),
                Cell::new(&config.updated),
            ]));
        }

        table.printstd();
    }

    Ok(())
}
