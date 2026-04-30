/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;
use ::rpc::forge as forgerpc;
use chrono::TimeZone;

use crate::managed_host::StartUpdates;
use crate::rpc::ApiClient;

pub async fn start_updates(
    api_client: &ApiClient,
    options: StartUpdates,
) -> color_eyre::Result<()> {
    let (start_timestamp, end_timestamp) = if options.cancel {
        (
            chrono::Utc.timestamp_opt(0, 0).unwrap(),
            chrono::Utc.timestamp_opt(0, 0).unwrap(),
        )
    } else {
        let start = if let Some(start) = options.start {
            if let Some(start) = time_parse(start.as_str()) {
                start
            } else {
                return Err(CarbideCliError::GenericError(
                    "Invalid time format for --start".to_string(),
                )
                .into());
            }
        } else {
            chrono::Utc::now()
        };
        let end = if let Some(end) = options.end {
            if let Some(end) = time_parse(end.as_str()) {
                end
            } else {
                return Err(CarbideCliError::GenericError(
                    "Invalid time format for --end".to_string(),
                )
                .into());
            }
        } else {
            start
                .checked_add_signed(chrono::TimeDelta::days(1))
                .unwrap()
        };
        (start, end)
    };
    let request = forgerpc::SetFirmwareUpdateTimeWindowRequest {
        machine_ids: options.machines,
        start_timestamp: Some(start_timestamp.into()),
        end_timestamp: Some(end_timestamp.into()),
    };
    api_client
        .0
        .set_firmware_update_time_window(request)
        .await?;
    println!("Request complete");
    Ok(())
}

fn time_parse(input: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    if let Ok(output) = chrono::DateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S%z") {
        Some(output.with_timezone(&chrono::Utc))
    } else if let Ok(output) = chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%dT%H:%M:%S") {
        chrono::Local
            .from_local_datetime(&output)
            .earliest()
            .map(|x| x.to_utc())
    } else {
        None
    }
}
