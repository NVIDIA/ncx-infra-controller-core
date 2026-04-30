/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use dns_record::SoaRecord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SoaSnapshot(pub dns_record::SoaRecord);

impl SoaSnapshot {
    pub fn new(domain: &str) -> Self {
        SoaSnapshot(SoaRecord::new(domain))
    }

    pub fn increment_serial(&mut self) {
        self.0.increment_serial();
    }
}
