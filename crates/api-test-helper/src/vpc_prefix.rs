/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::SocketAddr;

use super::grpcurl::grpcurl_id;

pub async fn create(
    carbide_api_addrs: &[SocketAddr],
    vpc_id: &str,
    prefix: &str,
    name: &str,
) -> eyre::Result<String> {
    tracing::info!("Creating VPC prefix {prefix} ({name})");

    let data = serde_json::json!({
        "vpc_id": { "value": vpc_id },
        "config": {
            "prefix": prefix,
        },
        "metadata": {
            "name": name,
            "description": format!("VPC prefix for {prefix}"),
        },
    });
    let prefix_id = grpcurl_id(carbide_api_addrs, "CreateVpcPrefix", &data.to_string()).await?;
    tracing::info!("VPC prefix created with ID {prefix_id}");
    Ok(prefix_id)
}
