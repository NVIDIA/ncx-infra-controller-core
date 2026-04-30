/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, OutputFormat};
use prettytable::{Cell, Row, Table};

use super::args::Args;
use crate::component_manager::common;
use crate::rpc::ApiClient;

pub async fn list_versions(
    opts: Args,
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let response = api_client
        .0
        .list_component_firmware_versions(opts)
        .await
        .map_err(CarbideCliError::from)?;

    if format == OutputFormat::Json {
        let devices = response
            .devices
            .iter()
            .map(|device| {
                serde_json::json!({
                    "result": common::component_result_json(device.result.as_ref()),
                    "versions": device.versions,
                })
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&devices)?);
    } else {
        let mut table = Table::new();
        table.set_titles(Row::new(vec![
            Cell::new("Component ID"),
            Cell::new("Result"),
            Cell::new("Versions"),
            Cell::new("Error"),
        ]));

        for device in &response.devices {
            let (component_id, result_status, error) =
                common::component_result_fields(device.result.as_ref());
            let versions = common::join_or_dash(&device.versions);
            table.add_row(Row::new(vec![
                Cell::new(&component_id),
                Cell::new(&result_status),
                Cell::new(&versions),
                Cell::new(&error),
            ]));
        }

        table.printstd();
    }

    let (failures, failure_summary) = common::component_failure_count_and_summary(
        response.devices.iter().map(|device| device.result.as_ref()),
    );

    if failures > 0 {
        return Err(CarbideCliError::GenericError(format!(
            "{failures} component firmware version result(s) failed{failure_summary}"
        )));
    }

    Ok(())
}
