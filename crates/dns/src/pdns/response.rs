/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdnsResponse {
    result: Value,
}

impl PdnsResponse {
    pub fn new(result: serde_json::Value) -> Self {
        PdnsResponse { result }
    }
}

impl From<Value> for PdnsResponse {
    fn from(value: Value) -> Self {
        PdnsResponse { result: value }
    }
}

impl From<Vec<Value>> for PdnsResponse {
    fn from(values: Vec<Value>) -> Self {
        PdnsResponse {
            result: Value::Array(values),
        }
    }
}
