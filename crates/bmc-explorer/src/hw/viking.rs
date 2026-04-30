/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

pub const EXPECTED_BIOS_ATTRS: [BiosAttr; 13] = [
    // Serial console enabled:
    BiosAttr::new_bool("AcpiSpcrConsoleRedirectionEnable", true),
    BiosAttr::new_bool("ConsoleRedirectionEnable0", true),
    BiosAttr::new_str("AcpiSpcrPort", "COM0"),
    BiosAttr::new_str("AcpiSpcrFlowControl", "None"),
    BiosAttr::new_str("AcpiSpcrBaudRate", "115200"),
    BiosAttr::new_str("BaudRate0", "115200"),
    // Virtualization enabled:
    BiosAttr::new_str("SriovSupport", "Enabled"), // sic! Enabled
    BiosAttr::new_str("VTdSupport", "Enable"),    // sic! Enable
    // HTTP/PXE:
    BiosAttr::new_str("Ipv4Http", "Enabled"),
    BiosAttr::new_str("Ipv4Pxe", "Disabled"),
    BiosAttr::new_str("Ipv6Http", "Enabled"),
    BiosAttr::new_str("Ipv6Pxe", "Disabled"),
    // Infinite boot:
    BiosAttr::new_str("NvidiaInfiniteboot", "Enable"), // sic! Enable
];
