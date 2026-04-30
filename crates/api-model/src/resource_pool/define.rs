/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct ResourcePoolDef {
    #[serde(default)]
    pub ranges: Vec<Range>,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(rename = "type")]
    pub pool_type: ResourcePoolType,
    #[serde(default)]
    pub delegate_prefix_len: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Range {
    pub start: String,
    pub end: String,
    #[serde(default = "default_true")]
    pub auto_assign: bool,
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourcePoolType {
    Ipv4,
    Ipv6,
    Ipv6Prefix,
    Integer,
}

fn default_true() -> bool {
    true
}
