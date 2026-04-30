/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::SocketAddr;

use super::grpcurl::grpcurl_id;

pub async fn create(carbide_api_addrs: &[SocketAddr], name: &str) -> eyre::Result<String> {
    tracing::info!("Creating domain");

    let data = serde_json::json!({
        "name": name,
    });
    let domain_id = grpcurl_id(carbide_api_addrs, "CreateDomain", &data.to_string()).await?;
    tracing::info!("Domain created with ID {domain_id}");
    Ok(domain_id)
}
