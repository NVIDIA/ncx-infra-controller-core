/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

pub const EXPECTED_BIOS_ATTRS: [BiosAttr; 5] = [
    // Serial console enabled:
    // Not implemented yet.
    //
    // Virtualization enabled:
    BiosAttr::new_str("IntelProcVtd", "Enabled"), // Intel
    BiosAttr::new_str("ProcAmdIoVt", "Enabled"),  // AMD
    BiosAttr::new_str("ProcVirtualization", "Enabled"), // iLO 7 Intel fallback
    // UEFI:
    BiosAttr::new_str("Dhcpv4", "Enabled"),
    BiosAttr::new_str("HttpSupport", "Auto"),
];
