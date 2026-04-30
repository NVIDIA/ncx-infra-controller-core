/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

pub const EXPECTED_BIOS_ATTRS: [BiosAttr; 4] = [
    BiosAttr::new_str("TPM", "Enabled"),
    BiosAttr::new_str("EmbeddedUefiShell", "Disabled"),
    // Enable Option ROM so that the DPU will show up in the Host's network device list
    // Otherwise, we will never see the DPU's Host PF MAC in the boot option list
    BiosAttr::new_bool("Socket0Pcie6DisableOptionROM", false),
    BiosAttr::new_bool("Socket1Pcie6DisableOptionROM", false),
];
