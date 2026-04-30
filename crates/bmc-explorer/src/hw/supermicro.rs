/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::hw::BiosAttr;

// Real attribute names examples:
// "IPv4HTTPSupport_009F", "DeviceSelect_0034", "DeviceSelect_003D", "SR_IOVSupport_002B"
// This constant defines prefixes till last underscore.
pub const EXPECTED_BIOS_ATTRS_PREFIXES: [BiosAttr; 14] = [
    BiosAttr::new_bool("QuietBoot", false),
    BiosAttr::new_str("Re_tryBoot", "EFI Boot"),
    BiosAttr::new_str("CSMSupport", "Disabled"),
    BiosAttr::new_bool("SecureBootEnable", false),
    // Trusted Computing / Provision Support / TXT Support
    BiosAttr::new_str("TXTSupport", "Enabled"),
    // registries/BiosAttributeRegistry.1.0.0.json/index.json
    BiosAttr::new_str("DeviceSelect", "TPM 2.0"),
    // Attributes to enable CPU virtualization support for faster VMs
    // Not that some are "Enable" and some are "Enabled". Subtle.
    BiosAttr::new_str("IntelVTforDirectedI_O_VT_d", "Enable"),
    BiosAttr::new_str("IntelVirtualizationTechnology", "Enable"),
    BiosAttr::new_str("SR_IOVSupport", "Enabled"),
    // UEFI NIC boot
    BiosAttr::new_str("IPv4HTTPSupport", "Enabled"),
    BiosAttr::new_str("IPv4PXESupport", "Disabled"),
    BiosAttr::new_str("IPv6HTTPSupport", "Disabled"),
    BiosAttr::new_str("IPv6PXESupport", "Disabled"),
    // TPM:
    BiosAttr::new_any_str("SecurityDeviceSupport", &["Enabled", "Enable"]),
];
