/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for ComputeAllocationId
pub struct ComputeAllocationIdMarker;

impl UuidSubtype for ComputeAllocationIdMarker {
    const TYPE_NAME: &'static str = "ComputeAllocationId";
}

/// ComputeAllocationId is a strongly typed UUID for ComputeAllocations.
pub type ComputeAllocationId = TypedUuid<ComputeAllocationIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    // Run all boilerplate TypedUuid tests for this type, also
    // ensuring TYPE_NAME and DB_COLUMN_NAME test correctly.
    typed_uuid_tests!(ComputeAllocationId, "ComputeAllocationId", "id");
}
