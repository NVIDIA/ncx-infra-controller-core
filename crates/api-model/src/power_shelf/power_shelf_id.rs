/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::power_shelf::{PowerShelfId, PowerShelfIdSource, PowerShelfType};
use sha2::{Digest, Sha256};

/// Generates a Power Shelf ID from the hardware fingerprint
///
/// Returns `None` if no sufficient data is available
pub fn from_hardware_info_with_type(
    serial: &str,
    vendor: &str,
    model: &str,
    source: PowerShelfIdSource,
    power_shelf_type: PowerShelfType,
) -> Result<PowerShelfId, MissingHardwareInfo> {
    let bytes = format!("s{}-b{}-c{}", serial, vendor, model);
    let mut hasher = Sha256::new();
    hasher.update(bytes.as_bytes());

    Ok(PowerShelfId::new(
        source,
        hasher.finalize().into(),
        power_shelf_type,
    ))
}

/// Generates a Power Shelf ID from a hardware fingerprint
pub fn from_hardware_info(
    serial: &str,
    vendor: &str,
    model: &str,
    source: PowerShelfIdSource,
    power_shelf_type: PowerShelfType,
) -> Result<PowerShelfId, MissingHardwareInfo> {
    from_hardware_info_with_type(serial, vendor, model, source, power_shelf_type)
}

#[derive(Debug, Copy, Clone, PartialEq, thiserror::Error)]
pub enum MissingHardwareInfo {
    #[error("The TPM certificate has no bytes")]
    TPMCertEmpty,
    #[error("Serial number missing (product, board and chassis)")]
    Serial,
    #[error("TPM and DMI data are both missing")]
    All,
}
