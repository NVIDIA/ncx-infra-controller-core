/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for InstanceId.
pub struct InstanceIdMarker;

impl UuidSubtype for InstanceIdMarker {
    const TYPE_NAME: &'static str = "InstanceId";
}

/// InstanceId is a strongly typed UUID specific to an instance ID.
pub type InstanceId = TypedUuid<InstanceIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    // Run all boilerplate TypedUuid tests for this type, also
    // ensuring TYPE_NAME and DB_COLUMN_NAME test correctly.
    typed_uuid_tests!(InstanceId, "InstanceId", "id");
}
