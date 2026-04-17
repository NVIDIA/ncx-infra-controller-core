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

use carbide_uuid::operating_system::OperatingSystemId;
use rpc::forge::{
    self, IpxeTemplateArtifactUpdateRequest, OperatingSystem, OperatingSystemsByIdsRequest,
    UpdateOperatingSystemIpxeTemplateArtifactRequest,
};
use rpc::forge_api_client::ForgeApiClient;

use crate::error::ImageCacheError;

#[derive(Clone)]
pub struct ApiClient(pub ForgeApiClient);

impl ApiClient {
    pub async fn discover_os_ids(
        &self,
        tenant_filter: Option<String>,
    ) -> Result<Vec<OperatingSystemId>, ImageCacheError> {
        let response = self
            .0
            .find_operating_system_ids(forge::OperatingSystemSearchFilter {
                tenant_organization_id: tenant_filter,
            })
            .await?;
        Ok(response.ids)
    }

    pub async fn get_os_definitions(
        &self,
        ids: Vec<OperatingSystemId>,
    ) -> Result<Vec<OperatingSystem>, ImageCacheError> {
        let response = self
            .0
            .find_operating_systems_by_ids(OperatingSystemsByIdsRequest { ids })
            .await?;
        Ok(response.operating_systems)
    }

    pub async fn set_artifact_cached_urls(
        &self,
        os_id: OperatingSystemId,
        updates: Vec<IpxeTemplateArtifactUpdateRequest>,
    ) -> Result<(), ImageCacheError> {
        self.0
            .update_operating_system_cachable_ipxe_template_artifacts(
                UpdateOperatingSystemIpxeTemplateArtifactRequest {
                    id: Some(os_id),
                    updates,
                },
            )
            .await?;
        Ok(())
    }
}
