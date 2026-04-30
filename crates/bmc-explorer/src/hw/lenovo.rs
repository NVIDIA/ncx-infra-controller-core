/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

pub const EXPECTED_BIOS_ATTRS: [BiosAttr; 14] = [
    // Serial console enabled:
    BiosAttr::new_str("DevicesandIOPorts_COMPort1", "Enabled"),
    BiosAttr::new_str("DevicesandIOPorts_ConsoleRedirection", "Enabled"),
    BiosAttr::new_str("DevicesandIOPorts_SerialPortSharing", "Enabled"),
    BiosAttr::new_str("DevicesandIOPorts_SPRedirection", "Enabled"),
    BiosAttr::new_str("DevicesandIOPorts_COMPortActiveAfterBoot", "Enabled"),
    BiosAttr::new_str("DevicesandIOPorts_SerialPortAccessMode", "Shared"),
    // Virtualization enabled:
    BiosAttr::new_str("Processors_IntelVirtualizationTechnology", "Enabled"), // Intel
    BiosAttr::new_str("Processors_SVMMode", "Enabled"),                       // AMD
    // UEFI:
    BiosAttr::new_str("BootModes_SystemBootMode", "UEFIMode"),
    BiosAttr::new_str("NetworkStackSettings_IPv4HTTPSupport", "Enabled"),
    BiosAttr::new_str("NetworkStackSettings_IPv4PXESupport", "Disabled"),
    BiosAttr::new_str("NetworkStackSettings_IPv6PXESupport", "Disabled"),
    BiosAttr::new_str("BootModes_InfiniteBootRetry", "Enabled"),
    BiosAttr::new_str("BootModes_PreventOSChangesToBootOrder", "Enabled"),
];
