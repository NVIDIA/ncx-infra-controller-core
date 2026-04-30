/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::SocketAddr;

use super::grpcurl::grpcurl_id;

pub async fn create(carbide_api_addrs: &[SocketAddr], tenant_org_id: &str) -> eyre::Result<String> {
    tracing::info!("Creating VPC");

    let data = serde_json::json!({
        "name": "tenant_vpc",
        "tenantOrganizationId": tenant_org_id,
        "routing_profile_type": "EXTERNAL".to_string(),
    });
    let vpc_id = grpcurl_id(carbide_api_addrs, "CreateVpc", &data.to_string()).await?;
    tracing::info!("VPC created with ID {vpc_id}");
    Ok(vpc_id)
}

pub async fn create_fnn(
    carbide_api_addrs: &[SocketAddr],
    tenant_org_id: &str,
) -> eyre::Result<String> {
    tracing::info!("Creating FNN VPC");

    let data = serde_json::json!({
        "name": "tenant_vpc_fnn",
        "tenantOrganizationId": tenant_org_id,
        "routing_profile_type": "EXTERNAL".to_string(),
        "network_virtualization_type": 5, // FNN
    });
    let vpc_id = grpcurl_id(carbide_api_addrs, "CreateVpc", &data.to_string()).await?;
    tracing::info!("FNN VPC created with ID {vpc_id}");
    Ok(vpc_id)
}
