/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// SLA for Switch initialization in seconds
pub const INITIALIZING: u64 = 300; // 5 minutes

/// SLA for Switch configuring in seconds
pub const CONFIGURING: u64 = 300; // 5 minutes

/// SLA for Switch validating in seconds
pub const VALIDATING: u64 = 300; // 5 minutes

// /// SLA for Switch ready in seconds
// pub const READY: u64 = 0; // 0 minutes

// /// SLA for Switch error in seconds
// pub const ERROR: u64 = 300; // 5 minutes

/// SLA for Switch deleting in seconds
pub const DELETING: u64 = 300; // 5 minutes
