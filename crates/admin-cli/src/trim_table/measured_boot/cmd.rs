/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn trim_measured_boot(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let request = ::rpc::forge::TrimTableRequest {
        target: ::rpc::forge::TrimTableTarget::MeasuredBoot.into(),
        keep_entries: args.keep_entries,
    };

    let response = api_client.0.trim_table(request).await?;

    println!(
        "Trimmed {} reports from Measured Boot",
        response.total_deleted
    );
    Ok(())
}
