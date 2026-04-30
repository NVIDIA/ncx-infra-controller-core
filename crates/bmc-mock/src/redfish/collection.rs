/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use serde_json::json;

use crate::json::{JsonExt, JsonPatch};

/// Defines minimal set of Redfish resource attributes.
pub struct Collection<'a> {
    pub odata_id: Cow<'a, str>,
    pub odata_type: Cow<'a, str>,
    pub name: Cow<'a, str>,
}

impl Collection<'_> {
    pub fn nav_property(&self, name: &str) -> serde_json::Value {
        json!({
            name: {
                "@odata.id": self.odata_id
            }
        })
    }

    pub fn with_members(&self, members: &[impl serde::Serialize]) -> serde_json::Value {
        let count = members.len();
        self.json_patch().patch(json!({
            "Members": members,
            "Members@odata.count": count,
        }))
    }
}

impl<'a> AsRef<Collection<'a>> for Collection<'a> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl JsonPatch for Collection<'_> {
    fn json_patch(&self) -> serde_json::Value {
        json!({
            "@odata.id": self.odata_id,
            "@odata.type": self.odata_type,
            "Name": self.name,
        })
    }
}
