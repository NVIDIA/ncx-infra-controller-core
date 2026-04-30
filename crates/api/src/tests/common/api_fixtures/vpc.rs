/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge as rpc;
use carbide_uuid::vpc::VpcId;
use rpc::forge_server::Forge;

use super::TestEnv;
use crate::tests::common::api_fixtures::instance::default_tenant_config;
use crate::tests::common::rpc_builder::VpcCreationRequest;

pub async fn create_vpc(
    env: &TestEnv,
    name: String,
    tenant_org_id: Option<String>,
    vpc_metadata: Option<rpc::Metadata>,
) -> (VpcId, rpc::Vpc) {
    let tenant_config = default_tenant_config();

    let vpc_id = VpcId::new();
    let request = VpcCreationRequest::builder(
        "",
        tenant_org_id.unwrap_or(tenant_config.tenant_organization_id),
    )
    .id(vpc_id)
    .metadata(rpc::Metadata {
        name,
        description: vpc_metadata
            .as_ref()
            .map_or("".to_string(), |s| s.description.clone()),
        labels: vpc_metadata
            .as_ref()
            .map_or(Vec::new(), |s| s.labels.clone()),
    })
    .tonic_request();

    let response = env.api.create_vpc(request).await;
    let vpc = response.unwrap().into_inner();

    (vpc_id, vpc)
}
