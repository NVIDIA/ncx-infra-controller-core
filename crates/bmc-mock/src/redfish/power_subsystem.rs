/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use crate::json::{JsonExt, JsonPatch};
use crate::redfish;
use crate::redfish::Builder;

pub fn resource(chassis_id: &str) -> redfish::Resource<'static> {
    let odata_id = format!(
        "{}/PowerSubsystem",
        redfish::chassis::resource(chassis_id).odata_id
    );
    redfish::Resource {
        odata_id: Cow::Owned(odata_id),
        odata_type: Cow::Borrowed("#PowerSubsystem.v1_1_3.PowerSubsystem"),
        id: Cow::Borrowed("PowerSubsystem"),
        name: Cow::Borrowed("Power Subsystem"),
    }
}

pub fn builder(resource: &redfish::Resource) -> PowerSubsystemBuilder {
    PowerSubsystemBuilder {
        value: resource.json_patch(),
    }
}

pub struct PowerSubsystemBuilder {
    value: serde_json::Value,
}

impl Builder for PowerSubsystemBuilder {
    fn apply_patch(self, patch: serde_json::Value) -> Self {
        Self {
            value: self.value.patch(patch),
        }
    }
}

impl PowerSubsystemBuilder {
    pub fn power_supplies(self, v: redfish::Collection) -> Self {
        self.apply_patch(v.nav_property("PowerSupplies"))
    }

    pub fn build(self) -> serde_json::Value {
        self.value
    }
}
