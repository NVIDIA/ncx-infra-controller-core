/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

pub const EXPECTED_BIOS_ATTRS: [BiosAttr; 10] = [
    BiosAttr::new_str("VMXEN", "Enable"), // VMX (Intel Virtualization)
    BiosAttr::new_str("PCIS007", "Enabled"), // SR-IOV Support
    BiosAttr::new_int("LEM0001", 3),      // PXE retry count (remove on future FW update)
    BiosAttr::new_str("NWSK000", "Enabled"), // Network Stack
    BiosAttr::new_str("NWSK001", "Disabled"), // IPv4 PXE Support
    BiosAttr::new_str("NWSK006", "Enabled"), // IPv4 HTTP Support
    BiosAttr::new_str("NWSK002", "Disabled"), // IPv6 PXE Support
    BiosAttr::new_str("NWSK007", "Disabled"), // IPv6 HTTP Support
    BiosAttr::new_str("FBO001", "UEFI"),  // Boot Mode Select
    BiosAttr::new_str("EndlessBoot", "Enabled"), // Infinite Boot
];
