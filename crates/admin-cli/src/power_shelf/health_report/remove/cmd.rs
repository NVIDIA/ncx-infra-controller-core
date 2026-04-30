/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::RemovePowerShelfHealthReportRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn remove(api_client: &ApiClient, args: Args) -> CarbideCliResult<()> {
    api_client
        .0
        .remove_power_shelf_health_report(RemovePowerShelfHealthReportRequest {
            power_shelf_id: Some(args.power_shelf_id),
            source: args.report_source,
        })
        .await?;

    Ok(())
}
