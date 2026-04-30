/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

pub const EXPECTED_BIOS_ATTRS: [BiosAttr; 8] = [
    BiosAttr::new_str("PCIS007", "PCIS007Enabled"), // SR-IOV Support
    BiosAttr::new_int("LEM0001", 3),                // PXE retry count
    BiosAttr::new_str("NWSK000", "NWSK000Enabled"), // Network Stack
    BiosAttr::new_str("NWSK001", "NWSK001Disabled"), // IPv4 PXE Support
    BiosAttr::new_str("NWSK006", "NWSK006Enabled"), // IPv4 HTTP Support
    BiosAttr::new_str("NWSK002", "NWSK002Disabled"), // IPv6 PXE Support
    BiosAttr::new_str("NWSK007", "NWSK007Disabled"), // IPv6 HTTP Support
    BiosAttr::new_int("LEM0003", 50),               // Infinite Boot
];
