/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for DomainId.
pub struct DomainIdMarker;

impl UuidSubtype for DomainIdMarker {
    const TYPE_NAME: &'static str = "DomainId";
}

/// DomainId is a strongly typed UUID specific to
/// an Infiniband domain ID.
pub type DomainId = TypedUuid<DomainIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    // Run all boilerplate TypedUuid tests for this type, also
    // ensuring TYPE_NAME and DB_COLUMN_NAME test correctly.
    typed_uuid_tests!(DomainId, "DomainId", "id");
}
