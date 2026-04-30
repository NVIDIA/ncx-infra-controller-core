/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::{RackId, RackProfileId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpectedRackJson {
    pub rack_id: RackId,
    pub rack_profile_id: RackProfileId,
    #[serde(default)]
    pub metadata: Option<rpc::forge::Metadata>,
}
