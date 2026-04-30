/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use mac_address::MacAddress;

use crate::hw;

// This type describes Intel® Ethernet Network Adapter I210.
pub struct NicIntelI210 {
    pub mac_address: MacAddress,
}

impl NicIntelI210 {
    pub fn ethernet_nic(&self) -> hw::nic::Nic<'static> {
        hw::nic::Nic {
            mac_address: self.mac_address,
            serial_number: None,
            manufacturer: None,
            model: None,
            description: None,
            part_number: None,
            firmware_version: None,
            is_mat_dpu: false,
        }
    }
}
