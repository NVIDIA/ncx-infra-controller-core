/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge as forgerpc;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn add_bmc(data: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client
        .0
        .create_credential(forgerpc::CredentialCreationRequest::try_from(data)?)
        .await?;
    Ok(())
}
