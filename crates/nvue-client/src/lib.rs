/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod client;
pub mod config;

pub use client::NvueClient;
pub use config::NvueConfig;

pub mod types {
    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct MacTableEntry {
        pub mac: String,
        pub interface: String,
        pub entry_type: String,
        pub vlan: Option<u16>,
    }
}
