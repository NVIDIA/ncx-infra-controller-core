/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use mac_address::MacAddress;

use crate::hw;

// This type describes Intel® Ethernet Network Adapter E810.
pub struct NicIntelE810 {
    pub mac_addresses: [MacAddress; 2],
}

impl NicIntelE810 {
    pub fn ethernet_nics(&self) -> [hw::nic::Nic<'static>; 2] {
        // Real serial numbers are MAC address of port0 without ':'.
        let serial_number = self.mac_addresses[0].to_string().replace(":", "");
        self.mac_addresses.map(|mac| hw::nic::Nic {
            mac_address: mac,
            serial_number: Some(serial_number.clone().into()),
            manufacturer: None,
            model: None,
            description: None,
            part_number: Some("K91258-010".into()),
            firmware_version: None,
            is_mat_dpu: false,
        })
    }
}
