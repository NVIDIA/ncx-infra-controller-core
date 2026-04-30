/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for DpaInterfaceId
pub struct DpaInterfaceIdMarker;

impl UuidSubtype for DpaInterfaceIdMarker {
    const TYPE_NAME: &'static str = "DpaInterfaceId";
}

/// DpaInterfaceId is a strongly typed UUID for DPA interfaces.
pub type DpaInterfaceId = TypedUuid<DpaInterfaceIdMarker>;

/// A constant representing a null/empty DPA interface ID.
pub const NULL_DPA_INTERFACE_ID: DpaInterfaceId = TypedUuid::nil();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    // Run all boilerplate TypedUuid tests for this type, also
    // ensuring TYPE_NAME and DB_COLUMN_NAME test correctly.
    typed_uuid_tests!(DpaInterfaceId, "DpaInterfaceId", "id");

    #[test]
    fn test_null_constant() {
        assert_eq!(NULL_DPA_INTERFACE_ID, DpaInterfaceId::default());
        assert_eq!(uuid::Uuid::from(NULL_DPA_INTERFACE_ID), uuid::Uuid::nil());
    }
}
