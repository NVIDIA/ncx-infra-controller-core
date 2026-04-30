/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! State Controller implementation for Switches.

pub mod bom_validating;
pub mod configuring;
pub mod context;
pub mod created;
pub mod deleting;
pub mod error_state;
pub mod handler;
pub mod initializing;
pub mod io;
pub mod ready;
pub mod reprovisioning;
pub mod validating;
