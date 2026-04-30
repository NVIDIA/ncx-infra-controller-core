/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn get(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let mbo = api_client.0.get_machine_boot_override(args).await?;

    tracing::info!(
        "{}",
        serde_json::to_string_pretty(&mbo).expect("Failed to serialize MachineBootOverride")
    );
    Ok(())
}
