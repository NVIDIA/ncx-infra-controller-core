/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for VpcPeeringId
pub struct VpcPeeringIdMarker;

impl UuidSubtype for VpcPeeringIdMarker {
    const TYPE_NAME: &'static str = "VpcPeeringId";
}

/// VpcPeeringId is a strongly typed UUID specific to a VPC peering relationship.
pub type VpcPeeringId = TypedUuid<VpcPeeringIdMarker>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(VpcPeeringId, "VpcPeeringId", "id");
}
