/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, OutputFormat};
use prettytable::{Cell, Row, Table};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn history(
    opts: Args,
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let result = api_client.0.get_rack_firmware_history(opts).await?;

    if format == OutputFormat::Json {
        // Flatten to map<rack_id, Vec<record>> for serialization
        let json_histories: std::collections::HashMap<
            &str,
            Vec<&rpc::forge::RackFirmwareHistoryRecord>,
        > = result
            .histories
            .iter()
            .map(|(rack_id, records)| (rack_id.as_str(), records.records.iter().collect()))
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_histories)?);
    } else if result.histories.is_empty() {
        println!("No rack firmware apply history found.");
    } else {
        let mut table = Table::new();
        table.set_titles(Row::new(vec![
            Cell::new("Rack ID"),
            Cell::new("Firmware ID"),
            Cell::new("Hardware Type"),
            Cell::new("Firmware Type"),
            Cell::new("Applied At"),
            Cell::new("Available"),
        ]));

        for (rack_id, records) in &result.histories {
            for record in &records.records {
                let hw_type = record
                    .rack_hardware_type
                    .as_ref()
                    .map(|t| t.value.as_str())
                    .unwrap_or("N/A");
                table.add_row(Row::new(vec![
                    Cell::new(rack_id),
                    Cell::new(&record.firmware_id),
                    Cell::new(hw_type),
                    Cell::new(&record.firmware_type),
                    Cell::new(&record.applied_at),
                    Cell::new(&record.firmware_available.to_string()),
                ]));
            }
        }

        table.printstd();
    }

    Ok(())
}
