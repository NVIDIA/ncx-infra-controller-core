/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::IpAddr;

use carbide_uuid::rack::RackId;
use mac_address::MacAddress;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpectedSwitchJson {
    pub bmc_mac_address: MacAddress,
    pub bmc_username: String,
    pub bmc_password: String,
    pub switch_serial_number: String,
    #[serde(default)]
    pub nvos_mac_addresses: Vec<MacAddress>,
    pub nvos_username: Option<String>,
    pub nvos_password: Option<String>,
    #[serde(default)]
    pub metadata: Option<rpc::forge::Metadata>,
    pub rack_id: Option<RackId>,
    pub bmc_ip_address: Option<IpAddr>,
    #[serde(default)]
    pub bmc_retain_credentials: Option<bool>,
}
