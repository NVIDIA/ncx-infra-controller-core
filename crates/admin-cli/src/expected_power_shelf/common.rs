/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::IpAddr;

use carbide_uuid::rack::RackId;
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpectedPowerShelfJson {
    pub bmc_mac_address: MacAddress,
    pub bmc_username: String,
    pub bmc_password: String,
    pub shelf_serial_number: String,
    #[serde(default)]
    pub metadata: Option<rpc::forge::Metadata>,
    pub host_name: Option<String>,
    pub rack_id: Option<RackId>,
    pub bmc_ip_address: Option<IpAddr>,
    #[serde(default)]
    pub bmc_retain_credentials: Option<bool>,
}
