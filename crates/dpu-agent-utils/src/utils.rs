/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge_tls_client::{ApiConfig, ForgeClientConfig, ForgeClientT, ForgeTlsClient};

// Forge Communication
pub async fn create_forge_client(
    forge_api: &str,
    client_config: &ForgeClientConfig,
) -> Result<ForgeClientT, eyre::Error> {
    match ForgeTlsClient::retry_build(&ApiConfig::new(forge_api, client_config)).await {
        Ok(client) => Ok(client),
        Err(err) => Err(eyre::eyre!(
            "Could not connect to Forge API server at {}: {err}",
            forge_api
        )),
    }
}
