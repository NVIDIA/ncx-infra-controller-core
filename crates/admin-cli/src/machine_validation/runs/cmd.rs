/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliResult, OutputFormat};
use ::rpc::forge as forgerpc;
use prettytable::{Table, row};

use super::args::ShowRunsOptions;
use crate::rpc::ApiClient;

pub async fn handle_runs_show(
    args: ShowRunsOptions,
    output_format: OutputFormat,
    api_client: &ApiClient,
    _page_size: usize,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;
    show_runs(is_json, api_client, args).await?;
    Ok(())
}

async fn show_runs(
    json: bool,
    api_client: &ApiClient,
    args: ShowRunsOptions,
) -> CarbideCliResult<()> {
    let runs = match api_client
        .get_machine_validation_runs(args.machine, args.history)
        .await
    {
        Ok(runs) => runs,
        Err(e) => return Err(e),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&runs)?);
    } else {
        convert_runs_to_nice_table(runs).printstd();
    }
    Ok(())
}

fn convert_runs_to_nice_table(runs: forgerpc::MachineValidationRunList) -> Box<Table> {
    let mut table = Table::new();

    table.set_titles(row![
        "Id",
        "MachineId",
        "StartTime",
        "EndTime",
        "Context",
        "State"
    ]);

    for run in runs.runs {
        let end_time = if let Some(run_end_time) = run.end_time {
            run_end_time.to_string()
        } else {
            "".to_string()
        };
        let status_state = run
            .status
            .unwrap_or_default()
            .machine_validation_state
            .unwrap_or(
                forgerpc::machine_validation_status::MachineValidationState::Completed(
                    forgerpc::machine_validation_status::MachineValidationCompleted::Success.into(),
                ),
            );
        table.add_row(row![
            run.validation_id.unwrap_or_default(),
            run.machine_id.unwrap_or_default(),
            run.start_time.unwrap_or_default(),
            end_time,
            run.context.unwrap_or_default(),
            format!("{:?}", status_state),
        ]);
    }

    table.into()
}
