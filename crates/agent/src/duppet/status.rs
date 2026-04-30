/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use serde::Serialize;

/// SyncStatus is a simple enum that stores whether
/// the target file was created, updated, or already
/// in sync.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    // The file was created with the expected contents and permissions.
    Created,
    // The file contents or permissions were updated.
    Updated,
    // The file contents and permissions were in sync.
    Unchanged,
}
