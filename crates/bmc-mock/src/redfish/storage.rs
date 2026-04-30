/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use crate::redfish;

pub fn system_collection(system_id: &str) -> redfish::Collection<'static> {
    let odata_id = format!("/redfish/v1/Systems/{system_id}/Storage");
    redfish::Collection {
        odata_id: Cow::Owned(odata_id),
        odata_type: Cow::Borrowed("#StorageCollection.StorageCollection"),
        name: Cow::Borrowed("Storage Collection"),
    }
}

pub fn system_resource<'a>(system_id: &str, storage_id: &'a str) -> redfish::Resource<'a> {
    let odata_id = format!("/redfish/v1/Systems/{system_id}/Storage/{storage_id}");
    redfish::Resource {
        odata_id: Cow::Owned(odata_id),
        odata_type: Cow::Borrowed("#Storage.v1_15_0.Storage"),
        name: Cow::Borrowed("Storage"),
        id: Cow::Borrowed(storage_id),
    }
}

pub struct Storage {
    pub id: Cow<'static, str>,
    pub value: serde_json::Value,
}
