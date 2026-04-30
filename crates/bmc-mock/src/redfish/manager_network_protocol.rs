/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use serde_json::json;

use crate::json::{JsonExt, JsonPatch};
use crate::redfish;
use crate::redfish::Builder;

pub fn manager_resource<'a>(manager_id: &'a str) -> redfish::Resource<'a> {
    let odata_id = format!("/redfish/v1/Managers/{manager_id}/NetworkProtocol");
    redfish::Resource {
        odata_id: Cow::Owned(odata_id),
        odata_type: Cow::Borrowed("#ManagerNetworkProtocol.v1_5_0.ManagerNetworkProtocol"),
        id: Cow::Borrowed("NetworkProtocol"),
        name: Cow::Borrowed("Manager Network Protocol"),
    }
}

/// Get builder of the network adapter.
pub fn builder(resource: &redfish::Resource) -> ManagerNetworkProtocolBuilder {
    ManagerNetworkProtocolBuilder {
        value: resource.json_patch(),
    }
}

pub struct ManagerNetworkProtocolBuilder {
    value: serde_json::Value,
}

impl Builder for ManagerNetworkProtocolBuilder {
    fn apply_patch(self, patch: serde_json::Value) -> Self {
        Self {
            value: self.value.patch(patch),
        }
    }
}

impl ManagerNetworkProtocolBuilder {
    pub fn ipmi_enabled(self, value: bool) -> Self {
        self.apply_patch(json!({"IPMI": { "ProtocolEnabled": value }}))
    }

    pub fn build(self) -> serde_json::Value {
        self.value
    }
}
