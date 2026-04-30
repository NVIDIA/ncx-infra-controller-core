/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use crate::rpc::ApiClient;

pub async fn delete(ca_cert_id: i32, api_client: &ApiClient) -> CarbideCliResult<()> {
    Ok(api_client.0.tpm_delete_ca_cert(ca_cert_id).await?)
}
