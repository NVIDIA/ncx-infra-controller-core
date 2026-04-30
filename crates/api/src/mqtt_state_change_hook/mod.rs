/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! MQTT state change hook for publishing ManagedHostState transitions.
//!
//! This module implements the AsyncAPI specification defined in `carbide.yaml`,
//! publishing state changes to `nico/v1/machine/{machineId}/state` over MQTT 3.1.1.

pub mod hook;
pub mod message;
pub mod metrics;
