/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use serde_json::json;

pub fn dell_nic_info(function_id: &str, slot: usize, serial_number: &str) -> serde_json::Value {
    json!({
        "Dell": {
            "@odata.type": "#DellOem.v1_3_0.DellOemResources",
            "DellNIC": {
                "Id": function_id,
                "SerialNumber": serial_number,
                "DeviceDescription": format!("NIC in Slot {} Port 1", slot)
            }
        }
    })
}
