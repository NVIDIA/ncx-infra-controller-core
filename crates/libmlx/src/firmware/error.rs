/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::path::PathBuf;

use thiserror::Error;

// FirmwareError is the error type for firmware management operations,
// encompassing source resolution, flashing, verification, and reset.
#[derive(Debug, Error)]
pub enum FirmwareError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Failed to resolve firmware from '{description}': {reason}")]
    SourceResolution { description: String, reason: String },

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("HTTP download failed: {0}")]
    HttpError(String),

    #[error("SSH transfer failed: {0}")]
    SshError(String),

    #[error("Flint error: {0}")]
    FlintError(#[from] crate::lockdown::error::MlxError),

    #[error("mlxconfig error: {0}")]
    MlxConfigError(#[from] crate::runner::error::MlxRunnerError),

    #[error("Firmware reset failed: {0}")]
    ResetFailed(String),

    #[error("Firmware verification failed: {0}")]
    VerificationFailed(String),

    #[error("mlxfwreset tool not found or not executable")]
    MlxFwResetNotFound,

    #[error("Permission denied - requires root privileges")]
    PermissionDenied,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Dry run - would have executed: {0}")]
    DryRun(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

// FirmwareResult is a result type alias for firmware operations.
pub type FirmwareResult<T> = Result<T, FirmwareError>;
