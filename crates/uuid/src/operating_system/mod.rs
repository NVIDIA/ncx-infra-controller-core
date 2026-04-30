/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for OperatingSystemId.
pub struct OperatingSystemIdMarker;

impl UuidSubtype for OperatingSystemIdMarker {
    const TYPE_NAME: &'static str = "OperatingSystemId";
}

/// OperatingSystemId is a strongly typed UUID specific to an operating system definition ID.
pub type OperatingSystemId = TypedUuid<OperatingSystemIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(OperatingSystemId, "OperatingSystemId", "id");
}
