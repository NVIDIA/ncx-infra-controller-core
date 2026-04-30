/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::{self as rpc, InsertPowerShelfHealthReportRequest};

use super::args::Args;
use crate::health_utils;
use crate::rpc::ApiClient;

pub async fn add(api_client: &ApiClient, args: Args) -> CarbideCliResult<()> {
    let report =
        health_utils::resolve_health_report(args.template, args.health_report, args.message)?;

    if args.print_only {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return Ok(());
    }

    let request = InsertPowerShelfHealthReportRequest {
        power_shelf_id: Some(args.power_shelf_id),
        health_report_entry: Some(rpc::HealthReportEntry {
            report: Some(report.into()),
            mode: if args.replace {
                rpc::HealthReportApplyMode::Replace
            } else {
                rpc::HealthReportApplyMode::Merge
            } as i32,
        }),
    };
    api_client
        .0
        .insert_power_shelf_health_report(request)
        .await?;

    Ok(())
}
