/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use serde::{Deserialize, Serialize};

/// Power action shared across Switch (NVSwitch) and PowerShelf backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerAction {
    On,
    GracefulShutdown,
    ForceOff,
    GracefulRestart,
    ForceRestart,
    AcPowercycle,
}

/// Firmware update lifecycle state shared across Switch and PowerShelf backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirmwareState {
    Unknown,
    Queued,
    InProgress,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

/// Updatable components of an Switch tray.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NvSwitchComponent {
    Bmc,
    Cpld,
    Bios,
    Nvos,
}

/// Updatable components of a PowerShelf.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PowerShelfComponent {
    Pmc,
    Psu,
}
