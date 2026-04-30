/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge as forgerpc;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete_ufm(data: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client
        .0
        .delete_credential(forgerpc::CredentialDeletionRequest::try_from(data)?)
        .await?;
    Ok(())
}
