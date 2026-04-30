/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliResult, OutputFormat};

use super::args::Args;
use crate::health_utils;
use crate::rpc::ApiClient;

pub async fn show(
    api_client: &ApiClient,
    args: Args,
    format: OutputFormat,
) -> CarbideCliResult<()> {
    let response = api_client
        .0
        .list_switch_health_reports(args.switch_id)
        .await?;
    health_utils::display_health_reports(response.health_report_entries, format)?;
    Ok(())
}
