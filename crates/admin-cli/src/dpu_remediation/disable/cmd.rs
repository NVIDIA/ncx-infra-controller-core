/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn disable_dpu_remediation(
    data: Args,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let id = data.id;
    api_client.0.disable_remediation(data).await?;

    tracing::info!("Disabled remediation with id: {:?}", id);
    Ok(())
}
