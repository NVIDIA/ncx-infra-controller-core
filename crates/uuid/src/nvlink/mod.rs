/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::typed_uuids::{TypedUuid, UuidSubtype};

/// Marker type for NvLinkPartitionId
pub struct NvLinkPartitionIdMarker;

impl UuidSubtype for NvLinkPartitionIdMarker {
    const TYPE_NAME: &'static str = "NvLinkPartitionId";
}

/// NvLinkPartitionId is a strongly typed UUID specific to an NvLink partition.
pub type NvLinkPartitionId = TypedUuid<NvLinkPartitionIdMarker>;

/// Marker type for NvLinkLogicalPartitionId
pub struct NvLinkLogicalPartitionIdMarker;

impl UuidSubtype for NvLinkLogicalPartitionIdMarker {
    const TYPE_NAME: &'static str = "NvLinkLogicalPartitionId";
}

/// NvLinkLogicalPartitionId is a strongly typed UUID for NvLink logical partitions.
pub type NvLinkLogicalPartitionId = TypedUuid<NvLinkLogicalPartitionIdMarker>;

/// Marker type for NvLinkDomainId
pub struct NvLinkDomainIdMarker;

impl UuidSubtype for NvLinkDomainIdMarker {
    const TYPE_NAME: &'static str = "NvLinkDomainId";
}

/// NvLinkDomainId is a strongly typed UUID for NvLink domains.
pub type NvLinkDomainId = TypedUuid<NvLinkDomainIdMarker>;

#[cfg(test)]
mod nvlink_partition_id_tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(NvLinkPartitionId, "NvLinkPartitionId", "id");
}

#[cfg(test)]
mod nvlink_logical_partition_id_tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(NvLinkLogicalPartitionId, "NvLinkLogicalPartitionId", "id");
}

#[cfg(test)]
mod nvlink_domain_id_tests {
    use super::*;
    use crate::typed_uuid_tests;
    typed_uuid_tests!(NvLinkDomainId, "NvLinkDomainId", "id");
}
