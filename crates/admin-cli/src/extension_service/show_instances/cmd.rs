/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::admin_cli::output::OutputFormat;
use ::rpc::forge::InstanceDpuExtensionServiceInfo;
use prettytable::{Table, row};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn handle_show_instances(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let response = api_client
        .0
        .find_instances_by_dpu_extension_service(args)
        .await?;

    if is_json {
        let instances_json: Vec<serde_json::Value> = response
            .instances
            .iter()
            .map(|i| {
                serde_json::json!({
                    "instance_id": i.instance_id,
                    "service_id": i.service_id,
                    "version": i.version,
                    "removing": i.removed,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&instances_json)?);
    } else {
        convert_instances_to_table(&response.instances).printstd();
    }

    Ok(())
}

fn convert_instances_to_table(instances: &[InstanceDpuExtensionServiceInfo]) -> Box<Table> {
    let mut table = Table::new();

    table.set_titles(row![
        "Instance ID",
        "Service ID",
        "Version",
        "Config Status",
    ]);

    for instance in instances {
        let status = if instance.removed.is_some() {
            "Removing"
        } else {
            "Active"
        };

        table.add_row(row![
            instance.instance_id,
            instance.service_id,
            instance.version,
            status,
        ]);
    }

    table.into()
}
