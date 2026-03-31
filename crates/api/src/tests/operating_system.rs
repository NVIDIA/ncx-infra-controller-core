/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use rpc::forge::forge_server::Forge;
use rpc::forge::{ArtifactCacheStrategy, IpxeOsArtifact, OperatingSystemType, TenantState};
use tonic::Code;

use crate::tests::common::api_fixtures::create_test_env;

#[crate::sqlx_test]
async fn test_create_operating_system_ipxe(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let resp = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "test-ipxe-os".to_string(),
            tenant_organization_id: "test-org".to_string(),
            description: Some("inline iPXE OS".to_string()),
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: Some("cloud-init data".to_string()),
            ipxe_script: Some("chain --autofree https://boot.netboot.xyz".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap();

    let os = resp.into_inner();
    assert_eq!(os.name, "test-ipxe-os");
    assert_eq!(os.tenant_organization_id, "test-org");
    assert_eq!(os.r#type, OperatingSystemType::OsTypeIpxe as i32);
    assert_eq!(os.description.as_deref(), Some("inline iPXE OS"));
    assert!(os.is_active);
    assert!(os.allow_override);
    assert!(!os.phone_home_enabled);
    assert_eq!(os.user_data.as_deref(), Some("cloud-init data"));
    assert_eq!(
        os.ipxe_script.as_deref(),
        Some("chain --autofree https://boot.netboot.xyz")
    );
    assert!(os.ipxe_template_name.is_none());
    assert!(os.id.is_some());
}

#[crate::sqlx_test]
async fn test_create_operating_system_requires_name(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let resp = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "".to_string(),
            tenant_organization_id: "test-org".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await;

    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().code(), Code::InvalidArgument);
}

#[crate::sqlx_test]
async fn test_create_operating_system_requires_variant(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let resp = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "test-os".to_string(),
            tenant_organization_id: "test-org".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: None,
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await;

    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().code(), Code::InvalidArgument);
}

#[crate::sqlx_test]
async fn test_get_operating_system(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let created = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "get-test-os".to_string(),
            tenant_organization_id: "org1".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://boot.example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    let id = created.id.clone().unwrap();

    let fetched = env
        .api
        .get_operating_system(tonic::Request::new(id))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "get-test-os");
    assert_eq!(fetched.r#type, OperatingSystemType::OsTypeIpxe as i32);
}

#[crate::sqlx_test]
async fn test_get_operating_system_not_found(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let id: rpc::common::Uuid = uuid::Uuid::nil().into();
    let resp = env
        .api
        .get_operating_system(tonic::Request::new(id))
        .await;

    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().code(), Code::NotFound);
}

#[crate::sqlx_test]
async fn test_update_operating_system(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let created = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "original-name".to_string(),
            tenant_organization_id: "org1".to_string(),
            description: Some("original desc".to_string()),
            is_active: true,
            allow_override: false,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    let id = created.id.clone().unwrap();

    let updated = env
        .api
        .update_operating_system(tonic::Request::new(rpc::forge::UpdateOperatingSystemRequest {
            id: Some(id),
            name: Some("updated-name".to_string()),
            description: Some("updated desc".to_string()),
            is_active: Some(false),
            allow_override: Some(true),
            phone_home_enabled: Some(true),
            user_data: Some("new user-data".to_string()),
            ipxe_script: Some("chain http://updated.example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: None,
            ipxe_artifacts: None,
            ipxe_definition_hash: None,
        }))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(updated.name, "updated-name");
    assert_eq!(updated.description.as_deref(), Some("updated desc"));
    assert!(!updated.is_active);
    assert!(updated.allow_override);
    assert!(updated.phone_home_enabled);
    assert_eq!(updated.user_data.as_deref(), Some("new user-data"));
    assert_eq!(
        updated.ipxe_script.as_deref(),
        Some("chain http://updated.example.com")
    );
}

#[crate::sqlx_test]
async fn test_delete_operating_system(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let created = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "delete-test-os".to_string(),
            tenant_organization_id: "org1".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    let id = created.id.clone().unwrap();

    let del_resp = env
        .api
        .delete_operating_system(tonic::Request::new(id.clone().into()))
        .await;
    assert!(del_resp.is_ok());

    let get_resp = env
        .api
        .get_operating_system(tonic::Request::new(id))
        .await;
    assert!(get_resp.is_err());
    assert_eq!(get_resp.unwrap_err().code(), Code::NotFound);
}

#[crate::sqlx_test]
async fn test_find_operating_system_ids(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let os1 = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "find-os-1".to_string(),
            tenant_organization_id: "find-org".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://one.example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    let os2 = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "find-os-2".to_string(),
            tenant_organization_id: "find-org".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://two.example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    let resp = env
        .api
        .find_operating_system_ids(tonic::Request::new(
            rpc::forge::OperatingSystemSearchFilter {
                tenant_organization_id: Some("find-org".to_string()),
            },
        ))
        .await
        .unwrap()
        .into_inner();

    let ids: Vec<String> = resp.ids.iter().map(|u| u.value.clone()).collect();
    let os1_id = os1.id.as_ref().unwrap().value.clone();
    let os2_id = os2.id.as_ref().unwrap().value.clone();
    assert!(ids.contains(&os1_id));
    assert!(ids.contains(&os2_id));
    assert_eq!(ids.len(), 2);
}

#[crate::sqlx_test]
async fn test_find_operating_systems_by_ids(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let os1 = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "by-id-os-1".to_string(),
            tenant_organization_id: "org1".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://one.example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    let id1 = os1.id.clone().unwrap();

    let resp = env
        .api
        .find_operating_systems_by_ids(tonic::Request::new(
            rpc::forge::OperatingSystemsByIdsRequest {
                ids: vec![id1],
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.operating_systems.len(), 1);
    assert_eq!(resp.operating_systems[0].name, "by-id-os-1");
}

#[crate::sqlx_test]
async fn test_list_ipxe_script_templates(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let resp = env
        .api
        .list_ipxe_script_templates(tonic::Request::new(
            rpc::forge::ListIpxeScriptTemplatesRequest {},
        ))
        .await
        .unwrap()
        .into_inner();

    assert!(
        !resp.templates.is_empty(),
        "should have at least one embedded template"
    );
    for tmpl in &resp.templates {
        assert!(!tmpl.name.is_empty());
        assert!(!tmpl.template.is_empty());
    }
}

#[crate::sqlx_test]
async fn test_get_ipxe_script_template(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let all = env
        .api
        .list_ipxe_script_templates(tonic::Request::new(
            rpc::forge::ListIpxeScriptTemplatesRequest {},
        ))
        .await
        .unwrap()
        .into_inner();

    let first_name = &all.templates[0].name;

    let resp = env
        .api
        .get_ipxe_script_template(tonic::Request::new(
            rpc::forge::GetIpxeScriptTemplateRequest {
                name: first_name.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(&resp.name, first_name);
    assert!(!resp.template.is_empty());
}

// ---------------------------------------------------------------------------
// Helpers shared by artifact tests
// ---------------------------------------------------------------------------

/// Creates an OS with a qcow-image template and two CACHED_ONLY artifacts plus
/// one CACHE_AS_NEEDED artifact, then returns the OS UUID string.
async fn create_os_with_artifacts(
    env: &crate::tests::common::api_fixtures::TestEnv,
) -> rpc::common::Uuid {
    let resp = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "artifact-test-os".to_string(),
            tenant_organization_id: "org1".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: None,
            ipxe_template_name: Some("qcow-image".to_string()),
            ipxe_parameters: vec![rpc::forge::IpxeOsParameter {
                name: "image_url".to_string(),
                value: "http://example.com/image.qcow2".to_string(),
            }],
            ipxe_artifacts: vec![
                IpxeOsArtifact {
                    name: "kernel".to_string(),
                    url: "http://example.com/kernel".to_string(),
                    sha: None,
                    auth_type: None,
                    auth_token: None,
                    cache_strategy: ArtifactCacheStrategy::CachedOnly as i32,
                    local_url: None,
                },
                IpxeOsArtifact {
                    name: "initrd".to_string(),
                    url: "http://example.com/initrd".to_string(),
                    sha: None,
                    auth_type: None,
                    auth_token: None,
                    cache_strategy: ArtifactCacheStrategy::CachedOnly as i32,
                    local_url: None,
                },
                IpxeOsArtifact {
                    name: "overlay".to_string(),
                    url: "http://example.com/overlay".to_string(),
                    sha: None,
                    auth_type: None,
                    auth_token: None,
                    cache_strategy: ArtifactCacheStrategy::CacheAsNeeded as i32,
                    local_url: None,
                },
            ],
        }))
        .await
        .unwrap()
        .into_inner();
    resp.id.unwrap()
}

// ---------------------------------------------------------------------------
// GetOperatingSystemArtifacts tests
// ---------------------------------------------------------------------------

#[crate::sqlx_test]
async fn test_get_operating_system_artifacts_returns_ordered_list(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;
    let os_id = create_os_with_artifacts(&env).await;

    let resp = env
        .api
        .get_operating_system_artifacts(tonic::Request::new(
            rpc::forge::GetOperatingSystemArtifactsRequest {
                id: Some(os_id),
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.artifacts.len(), 3);
    assert_eq!(resp.artifacts[0].name, "kernel");
    assert_eq!(resp.artifacts[1].name, "initrd");
    assert_eq!(resp.artifacts[2].name, "overlay");
}

#[crate::sqlx_test]
async fn test_get_operating_system_artifacts_not_found(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;
    let id: rpc::common::Uuid = uuid::Uuid::nil().into();

    let resp = env
        .api
        .get_operating_system_artifacts(tonic::Request::new(
            rpc::forge::GetOperatingSystemArtifactsRequest { id: Some(id) },
        ))
        .await;

    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().code(), Code::NotFound);
}

// ---------------------------------------------------------------------------
// SetOperatingSystemArtifactsLocalUrl tests
// ---------------------------------------------------------------------------

#[crate::sqlx_test]
async fn test_set_artifacts_local_url_partial_update(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;
    let os_id = create_os_with_artifacts(&env).await;

    // Only update the first artifact (partial list).
    let resp = env
        .api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id.clone()),
                updates: vec![rpc::forge::ArtifactLocalUrlUpdate {
                    name: "kernel".to_string(),
                    local_url: Some("http://cache.local/kernel".to_string()),
                }],
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.artifacts.len(), 3);
    assert_eq!(
        resp.artifacts[0].local_url.as_deref(),
        Some("http://cache.local/kernel")
    );
    assert!(resp.artifacts[1].local_url.is_none()); // initrd unchanged
    assert!(resp.artifacts[2].local_url.is_none()); // overlay unchanged
}

#[crate::sqlx_test]
async fn test_set_artifacts_local_url_ordered_duplicate_names(pool: sqlx::PgPool) {
    // OS has two artifacts named "kernel". Updates must appear twice to set both.
    let env = create_test_env(pool).await;

    let os = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "dup-kernel-os".to_string(),
            tenant_organization_id: "org1".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: None,
            ipxe_template_name: Some("qcow-image".to_string()),
            ipxe_parameters: vec![rpc::forge::IpxeOsParameter {
                name: "image_url".to_string(),
                value: "http://example.com/image.qcow2".to_string(),
            }],
            ipxe_artifacts: vec![
                IpxeOsArtifact {
                    name: "kernel".to_string(),
                    url: "http://example.com/kernel-a".to_string(),
                    sha: None,
                    auth_type: None,
                    auth_token: None,
                    cache_strategy: ArtifactCacheStrategy::CachedOnly as i32,
                    local_url: None,
                },
                IpxeOsArtifact {
                    name: "kernel".to_string(),
                    url: "http://example.com/kernel-b".to_string(),
                    sha: None,
                    auth_type: None,
                    auth_token: None,
                    cache_strategy: ArtifactCacheStrategy::CachedOnly as i32,
                    local_url: None,
                },
            ],
        }))
        .await
        .unwrap()
        .into_inner();

    let os_id = os.id.unwrap();

    // Two updates for "kernel" — each consumes the next unmatched occurrence.
    let resp = env
        .api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id.clone()),
                updates: vec![
                    rpc::forge::ArtifactLocalUrlUpdate {
                        name: "kernel".to_string(),
                        local_url: Some("http://cache.local/kernel-a".to_string()),
                    },
                    rpc::forge::ArtifactLocalUrlUpdate {
                        name: "kernel".to_string(),
                        local_url: Some("http://cache.local/kernel-b".to_string()),
                    },
                ],
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.artifacts[0].local_url.as_deref(), Some("http://cache.local/kernel-a"));
    assert_eq!(resp.artifacts[1].local_url.as_deref(), Some("http://cache.local/kernel-b"));
}

#[crate::sqlx_test]
async fn test_set_artifacts_local_url_too_many_same_name_fails(pool: sqlx::PgPool) {
    // Only one "kernel" in the OS but two update entries → second should fail.
    let env = create_test_env(pool).await;
    let os_id = create_os_with_artifacts(&env).await;

    let resp = env
        .api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id),
                updates: vec![
                    rpc::forge::ArtifactLocalUrlUpdate {
                        name: "kernel".to_string(),
                        local_url: Some("http://cache.local/kernel-1".to_string()),
                    },
                    rpc::forge::ArtifactLocalUrlUpdate {
                        name: "kernel".to_string(), // no second kernel exists
                        local_url: Some("http://cache.local/kernel-2".to_string()),
                    },
                ],
            },
        ))
        .await;

    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().code(), Code::NotFound);
}

#[crate::sqlx_test]
async fn test_set_artifacts_local_url_unknown_name_fails(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;
    let os_id = create_os_with_artifacts(&env).await;

    let resp = env
        .api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id),
                updates: vec![rpc::forge::ArtifactLocalUrlUpdate {
                    name: "does-not-exist".to_string(),
                    local_url: Some("http://cache.local/whatever".to_string()),
                }],
            },
        ))
        .await;

    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().code(), Code::NotFound);
}

#[crate::sqlx_test]
async fn test_set_artifacts_transitions_to_ready_when_all_cached_only_set(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;
    let os_id = create_os_with_artifacts(&env).await;

    // Set local_url only for the two CACHED_ONLY artifacts; leave the
    // CACHE_AS_NEEDED "overlay" artifact untouched.
    let _ = env
        .api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id.clone()),
                updates: vec![
                    rpc::forge::ArtifactLocalUrlUpdate {
                        name: "kernel".to_string(),
                        local_url: Some("http://cache.local/kernel".to_string()),
                    },
                    rpc::forge::ArtifactLocalUrlUpdate {
                        name: "initrd".to_string(),
                        local_url: Some("http://cache.local/initrd".to_string()),
                    },
                ],
            },
        ))
        .await
        .unwrap();

    // Status should now be READY.
    let fetched = env
        .api
        .get_operating_system(tonic::Request::new(os_id))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(fetched.status, TenantState::Ready as i32);
}

#[crate::sqlx_test]
async fn test_set_artifacts_does_not_transition_to_ready_when_cached_only_incomplete(
    pool: sqlx::PgPool,
) {
    let env = create_test_env(pool).await;
    let os_id = create_os_with_artifacts(&env).await;

    // Only set kernel; initrd (also CACHED_ONLY) is still missing.
    let _ = env
        .api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id.clone()),
                updates: vec![rpc::forge::ArtifactLocalUrlUpdate {
                    name: "kernel".to_string(),
                    local_url: Some("http://cache.local/kernel".to_string()),
                }],
            },
        ))
        .await
        .unwrap();

    let fetched = env
        .api
        .get_operating_system(tonic::Request::new(os_id))
        .await
        .unwrap()
        .into_inner();

    assert_ne!(fetched.status, TenantState::Ready as i32);
}

#[crate::sqlx_test]
async fn test_set_artifacts_local_url_clear(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;
    let os_id = create_os_with_artifacts(&env).await;

    // Set then clear kernel's local_url.
    env.api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id.clone()),
                updates: vec![rpc::forge::ArtifactLocalUrlUpdate {
                    name: "kernel".to_string(),
                    local_url: Some("http://cache.local/kernel".to_string()),
                }],
            },
        ))
        .await
        .unwrap();

    let resp = env
        .api
        .set_operating_system_artifacts_local_url(tonic::Request::new(
            rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(os_id),
                updates: vec![rpc::forge::ArtifactLocalUrlUpdate {
                    name: "kernel".to_string(),
                    local_url: None, // clear it
                }],
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert!(resp.artifacts[0].local_url.is_none());
}

#[crate::sqlx_test]
async fn test_get_ipxe_script_template_not_found(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let resp = env
        .api
        .get_ipxe_script_template(tonic::Request::new(
            rpc::forge::GetIpxeScriptTemplateRequest {
                name: "nonexistent-template".to_string(),
            },
        ))
        .await;

    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().code(), Code::NotFound);
}

#[crate::sqlx_test]
async fn test_create_operating_system_with_explicit_id(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let explicit_id = uuid::Uuid::new_v4();
    let id_proto: rpc::common::Uuid = explicit_id.into();

    let resp = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: Some(id_proto),
            name: "explicit-id-os".to_string(),
            tenant_organization_id: "org1".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.id.unwrap().value, explicit_id.to_string());
}

#[crate::sqlx_test]
async fn test_deleted_os_not_returned_by_find_ids(pool: sqlx::PgPool) {
    let env = create_test_env(pool).await;

    let created = env
        .api
        .create_operating_system(tonic::Request::new(rpc::forge::CreateOperatingSystemRequest {
            id: None,
            name: "soon-deleted-os".to_string(),
            tenant_organization_id: "del-org".to_string(),
            description: None,
            is_active: true,
            allow_override: true,
            phone_home_enabled: false,
            user_data: None,
            ipxe_script: Some("chain http://example.com".to_string()),
            ipxe_template_name: None,
            ipxe_parameters: vec![],
            ipxe_artifacts: vec![],
        }))
        .await
        .unwrap()
        .into_inner();

    let id = created.id.clone().unwrap();
    env.api
        .delete_operating_system(tonic::Request::new(id.into()))
        .await
        .unwrap();

    let resp = env
        .api
        .find_operating_system_ids(tonic::Request::new(
            rpc::forge::OperatingSystemSearchFilter {
                tenant_organization_id: Some("del-org".to_string()),
            },
        ))
        .await
        .unwrap()
        .into_inner();

    assert!(resp.ids.is_empty());
}
