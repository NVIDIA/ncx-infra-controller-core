/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;
use rpc::forge::CreateRemediationRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn create_dpu_remediation(
    create_remediation: Args,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let script = tokio::fs::read_to_string(&create_remediation.script_filename)
        .await
        .map_err(|err| {
            tracing::error!("Error reading script file for dpu remediation: {:?}", err);
            CarbideCliError::IOError(err)
        })?;

    let response = api_client
        .0
        .create_remediation(CreateRemediationRequest {
            script,
            retries: create_remediation.retries.unwrap_or_default() as i32,
            metadata: create_remediation.into_metadata(),
        })
        .await?;

    tracing::info!("Created remediation with id: {:?}", response.remediation_id);
    Ok(())
}
