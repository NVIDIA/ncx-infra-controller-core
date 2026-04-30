/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Common functions for GB200 and GB300.

use std::borrow::Cow;

use serde_json::json;

use crate::redfish;

pub struct Topology {
    pub chassis_physical_slot_number: u32,
    pub compute_tray_index: u32,
    pub revision_id: u32,
    pub topology_id: u32,
}

// CBC chassis definition.
pub fn cbc_chassis(
    chassis_id: Cow<'static, str>,
    topology: &Topology,
) -> redfish::chassis::SingleChassisConfig {
    redfish::chassis::SingleChassisConfig {
        id: chassis_id,
        chassis_type: "Component".into(),
        manufacturer: Some("Nvidia".into()),
        part_number: Some("750-0567-002".into()),
        model: Some("18x1RU CBL Cartridge".into()),
        serial_number: Some("1821220000000".into()),
        pcie_devices: Some(vec![]),
        oem: Some(json!({
            "Nvidia": {
                "@odata.type": "#NvidiaChassis.v1_4_0.NvidiaCBCChassis",
                "ChassisPhysicalSlotNumber": topology.chassis_physical_slot_number,
                "ComputeTrayIndex": topology.compute_tray_index,
                "RevisionId": topology.revision_id,
                "TopologyId": topology.topology_id,
            }
        })),
        ..redfish::chassis::SingleChassisConfig::defaults()
    }
}
