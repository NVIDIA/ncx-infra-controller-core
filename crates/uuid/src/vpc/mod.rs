/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for VpcId
pub struct VpcIdMarker;

impl UuidSubtype for VpcIdMarker {
    const TYPE_NAME: &'static str = "VpcId";
}

/// VpcId is a strongly typed UUID specific to a VPC ID, with
/// trait implementations allowing it to be passed around as
/// a UUID, an RPC UUID, bound to sqlx queries, etc.
pub type VpcId = TypedUuid<VpcIdMarker>;

/// Marker type for VpcPrefixId
pub struct VpcPrefixMarker;

impl UuidSubtype for VpcPrefixMarker {
    const TYPE_NAME: &'static str = "VpcPrefixId";
}

pub type VpcPrefixId = TypedUuid<VpcPrefixMarker>;

#[cfg(test)]
mod vpc_id_tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(VpcId, "VpcId", "id");
}

#[cfg(test)]
mod vpc_prefix_id_tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(VpcPrefixId, "VpcPrefixId", "id");
}
