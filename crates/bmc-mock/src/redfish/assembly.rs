/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use serde_json::json;

use crate::json::{JsonExt, JsonPatch};
use crate::redfish;
use crate::redfish::Builder;

pub fn chassis_resource(chassis_id: &str) -> redfish::Resource<'static> {
    let odata_id = format!("/redfish/v1/Chassis/{chassis_id}/Assembly");
    redfish::Resource {
        odata_id: odata_id.into(),
        odata_type: "#Assembly.v1_3_0.Assembly".into(),
        id: "Assembly".into(),
        name: format!("Assembly data for {chassis_id}").into(),
    }
}

pub fn builder(resource: &redfish::Resource) -> AssemblyBuilder {
    AssemblyBuilder {
        odata_id: resource.odata_id.to_string(),
        assemblies: vec![],
        value: resource.json_patch(),
    }
}

pub fn data_builder(member_id: Cow<'static, str>) -> AssemblyData {
    AssemblyData {
        member_id,
        value: json!({}),
    }
}

pub struct AssemblyBuilder {
    odata_id: String,
    assemblies: Vec<AssemblyData>,
    value: serde_json::Value,
}

impl Builder for AssemblyBuilder {
    fn apply_patch(self, patch: serde_json::Value) -> Self {
        Self {
            odata_id: self.odata_id,
            assemblies: self.assemblies,
            value: self.value.patch(patch),
        }
    }
}

impl AssemblyBuilder {
    pub fn add_data(mut self, data: AssemblyData) -> Self {
        self.assemblies.push(data);
        self
    }

    pub fn build(self) -> serde_json::Value {
        json!({
            "Assemblies":
            self.assemblies.into_iter().map(|assembly| {
                json!({
                    "@odata.id": format!("{}#/Assemblies/{}", self.odata_id, assembly.member_id),
                    "MemberId": assembly.member_id,
                }).patch(assembly.value)
            }).collect::<Vec<_>>()
        })
        .patch(self.value)
    }
}

pub struct AssemblyData {
    member_id: Cow<'static, str>,
    value: serde_json::Value,
}

impl Builder for AssemblyData {
    fn apply_patch(self, patch: serde_json::Value) -> Self {
        Self {
            member_id: self.member_id,
            value: self.value.patch(patch),
        }
    }
}

impl AssemblyData {
    pub fn serial_number(self, v: &str) -> Self {
        self.add_str_field("SerialNumber", v)
    }

    pub fn build(self) -> Self {
        self
    }
}
