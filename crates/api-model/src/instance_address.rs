/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::instance::InstanceId;
use carbide_uuid::network::NetworkSegmentId;
use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct InstanceAddress {
    pub instance_id: InstanceId,
    pub segment_id: NetworkSegmentId,
    // pub id: Uuid,          // unused
    pub address: std::net::IpAddr,
    // pub prefix: IpNetwork, // unused
}
