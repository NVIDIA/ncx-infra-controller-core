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

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use ::rpc::forge::{
    self as forgerpc, IpxeScriptArtifact, IpxeScriptParameter, OperatingSystemDefinition,
};
use serde::Serialize;

pub fn str_to_os_id(
    id: &str,
) -> CarbideCliResult<::carbide_uuid::operating_system::OperatingSystemId> {
    let id = uuid::Uuid::parse_str(id)
        .map_err(|e| CarbideCliError::GenericError(e.to_string()))?
        .into();
    Ok(id)
}

/// Parse a "key=value" string into an `IpxeScriptParameter`.
pub fn parse_param(s: &str) -> Result<IpxeScriptParameter, String> {
    let (name, value) = s
        .split_once('=')
        .ok_or_else(|| format!("expected KEY=VALUE, got '{s}'"))?;
    Ok(IpxeScriptParameter {
        name: name.to_string(),
        value: value.to_string(),
    })
}

/// Local serializable mirror of `OperatingSystemDefinition` for JSON output.
#[derive(Serialize)]
pub struct SerializableOs {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub org: String,
    #[serde(rename = "type")]
    pub os_type: String,
    pub status: String,
    pub is_active: bool,
    pub allow_override: bool,
    pub phone_home_enabled: bool,
    pub user_data: Option<String>,
    pub created: String,
    pub updated: String,
    pub ipxe_script: Option<String>,
    pub ipxe_template_name: Option<String>,
    pub ipxe_parameters: Vec<SerializableParam>,
    pub ipxe_artifacts: Vec<SerializableArtifact>,
    pub ipxe_definition_hash: Option<String>,
}

#[derive(Serialize)]
pub struct SerializableParam {
    pub name: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct SerializableArtifact {
    pub name: String,
    pub url: String,
    pub sha: Option<String>,
    pub auth_type: Option<String>,
    pub cache_strategy: String,
    pub cached_url: Option<String>,
}

impl From<IpxeScriptParameter> for SerializableParam {
    fn from(p: IpxeScriptParameter) -> Self {
        Self {
            name: p.name,
            value: p.value,
        }
    }
}

impl From<IpxeScriptArtifact> for SerializableArtifact {
    fn from(a: IpxeScriptArtifact) -> Self {
        use ::rpc::forge::IpxeScriptArtifactCacheStrategy;
        let cache_strategy = match IpxeScriptArtifactCacheStrategy::try_from(a.cache_strategy) {
            Ok(IpxeScriptArtifactCacheStrategy::CacheAsNeeded) => "cache_as_needed",
            Ok(IpxeScriptArtifactCacheStrategy::LocalOnly) => "local_only",
            Ok(IpxeScriptArtifactCacheStrategy::CachedOnly) => "cached_only",
            Ok(IpxeScriptArtifactCacheStrategy::RemoteOnly) => "remote_only",
            _ => "cache_as_needed",
        };
        Self {
            name: a.name,
            url: a.url,
            sha: a.sha,
            auth_type: a.auth_type,
            cache_strategy: cache_strategy.to_string(),
            cached_url: a.cached_url,
        }
    }
}

impl From<OperatingSystemDefinition> for SerializableOs {
    fn from(os: OperatingSystemDefinition) -> Self {
        Self {
            id: os.id.map(|u| u.to_string()).unwrap_or_default(),
            name: os.name,
            description: os.description,
            org: os.tenant_organization_id,
            os_type: forgerpc::OperatingSystemType::try_from(os.r#type)
                .map(|t| t.as_str_name().to_string())
                .unwrap_or_else(|_| os.r#type.to_string()),
            status: forgerpc::TenantState::try_from(os.status)
                .map(|s| s.as_str_name().to_string())
                .unwrap_or_else(|_| os.status.to_string()),
            is_active: os.is_active,
            allow_override: os.allow_override,
            phone_home_enabled: os.phone_home_enabled,
            user_data: os.user_data,
            created: os.created,
            updated: os.updated,
            ipxe_script: os.ipxe_script,
            ipxe_template_name: os.ipxe_template_name,
            ipxe_parameters: os.ipxe_parameters.into_iter().map(Into::into).collect(),
            ipxe_artifacts: os.ipxe_artifacts.into_iter().map(Into::into).collect(),
            ipxe_definition_hash: os.ipxe_definition_hash,
        }
    }
}
