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

//! Clears the cached_url on every artifact of every operating system definition.
//! Useful for resetting state during testing so the imagecache service will
//! re-download and re-cache all artifacts on its next cycle.
//!
//! Required environment variables:
//!   CARBIDE_API_INTERNAL_URL  (or defaults to https://carbide-api.forge-system.svc.cluster.local:1079)
//!   FORGE_ROOT_CAFILE_PATH
//!   FORGE_CLIENT_CERT_PATH
//!   FORGE_CLIENT_KEY_PATH

use std::env;

use forge_tls::client_config::ClientCert;
use rpc::forge::{
    IpxeTemplateArtifactUpdateRequest, OperatingSystemSearchFilter, OperatingSystemsByIdsRequest,
    UpdateOperatingSystemIpxeTemplateArtifactRequest,
};
use rpc::forge_api_client::ForgeApiClient;
use rpc::forge_tls_client::{ApiConfig, ForgeClientConfig};

fn env_required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} must be set"))
}

pub async fn clear_urls() -> Result<(), Box<dyn std::error::Error>> {
    let internal_api_url = env::var("CARBIDE_API_INTERNAL_URL")
        .unwrap_or_else(|_| "https://carbide-api.forge-system.svc.cluster.local:1079".to_string());

    let client_cert = Some(ClientCert {
        cert_path: env_required("FORGE_CLIENT_CERT_PATH"),
        key_path: env_required("FORGE_CLIENT_KEY_PATH"),
    });
    let client_config = ForgeClientConfig::new(env_required("FORGE_ROOT_CAFILE_PATH"), client_cert);

    let api = ForgeApiClient::new(&ApiConfig::new(&internal_api_url, &client_config));

    // Discover all OS definition IDs
    let os_ids = api
        .find_operating_system_ids(OperatingSystemSearchFilter {
            tenant_organization_id: None,
        })
        .await?;

    if os_ids.ids.is_empty() {
        println!("No operating system definitions found.");
        return Ok(());
    }

    println!("Found {} operating system definitions.", os_ids.ids.len());

    // Fetch full definitions
    let os_defs = api
        .find_operating_systems_by_ids(OperatingSystemsByIdsRequest { ids: os_ids.ids })
        .await?;

    let mut cleared = 0u32;
    for os_def in &os_defs.operating_systems {
        let os_id = match &os_def.id {
            Some(id) => *id,
            None => continue,
        };

        let updates: Vec<IpxeTemplateArtifactUpdateRequest> = os_def
            .ipxe_template_artifacts
            .iter()
            .filter(|a| a.cached_url.as_ref().is_some_and(|u| !u.is_empty()))
            .map(|a| IpxeTemplateArtifactUpdateRequest {
                name: a.name.clone(),
                cached_url: None,
            })
            .collect();

        if updates.is_empty() {
            continue;
        }

        let count = updates.len();
        api.update_operating_system_cachable_ipxe_template_artifacts(
            UpdateOperatingSystemIpxeTemplateArtifactRequest {
                id: Some(os_id),
                updates,
            },
        )
        .await?;

        println!("  {} — cleared {} artifact(s)", os_def.name, count);
        cleared += count as u32;
    }

    println!("Done. Cleared {cleared} artifact local URL(s).");
    Ok(())
}
