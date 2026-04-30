/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for IpxeTemplateId.
pub struct IpxeTemplateIdMarker;

impl UuidSubtype for IpxeTemplateIdMarker {
    const TYPE_NAME: &'static str = "IpxeTemplateId";
}

/// IpxeTemplateId is a strongly typed UUID specific to an iPXE script template.
pub type IpxeTemplateId = TypedUuid<IpxeTemplateIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(IpxeTemplateId, "IpxeTemplateId", "id");
}
