/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, OutputFormat};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn get_job_status(
    opts: Args,
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let response = api_client
        .0
        .get_rack_firmware_job_status(opts)
        .await
        .map_err(CarbideCliError::from)?;

    if format == OutputFormat::Json {
        let result = serde_json::json!({
            "job_id": response.job_id,
            "state": response.state,
            "state_description": response.state_description,
            "rack_id": response.rack_id,
            "node_id": response.node_id,
            "error_message": response.error_message,
            "result_json": response.result_json,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Firmware Job Status");
        println!("  Job ID:      {}", response.job_id);
        println!("  State:       {}", response.state);
        println!("  Description: {}", response.state_description);
        println!("  Rack:        {}", response.rack_id);
        println!("  Node:        {}", response.node_id);

        if !response.error_message.is_empty() {
            println!("  Error:       {}", response.error_message);
        }

        if !response.result_json.is_empty() {
            println!("  Result:      {}", response.result_json);
        }
    }

    Ok(())
}
