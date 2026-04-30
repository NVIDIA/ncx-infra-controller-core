/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for ExtensionServiceId.
pub struct ExtensionServiceIdMarker;

impl UuidSubtype for ExtensionServiceIdMarker {
    const TYPE_NAME: &'static str = "ExtensionServiceId";
}

/// ExtensionServiceId is a strongly typed UUID specific to an
/// extension service.
pub type ExtensionServiceId = TypedUuid<ExtensionServiceIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    // Run all boilerplate TypedUuid tests for this type, also
    // ensuring TYPE_NAME and DB_COLUMN_NAME test correctly.
    typed_uuid_tests!(ExtensionServiceId, "ExtensionServiceId", "id");
}
