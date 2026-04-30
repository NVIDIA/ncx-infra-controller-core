/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for IBPartitionId.
pub struct IBPartitionIdMarker;

impl UuidSubtype for IBPartitionIdMarker {
    const TYPE_NAME: &'static str = "IBPartitionId";
}

/// IBPartitionId is a strongly typed UUID specific to an
/// Infiniband partition ID.
pub type IBPartitionId = TypedUuid<IBPartitionIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    // Run all boilerplate TypedUuid tests for this type, also
    // ensuring TYPE_NAME and DB_COLUMN_NAME test correctly.
    typed_uuid_tests!(IBPartitionId, "IBPartitionId", "id");
}
