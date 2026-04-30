/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

pub const EXPECTED_BIOS_ATTRS: [BiosAttr; 4] = [
    BiosAttr::new_str("HostPrivilegeLevel", "Restricted"),
    BiosAttr::new_str("Host Privilege Level", "Restricted"), // Older version of this option.
    BiosAttr::new_str("InternalCPUModel", "Embedded"),
    BiosAttr::new_str("Internal CPU Model", "Embedded"), // Older version of this option.
];
