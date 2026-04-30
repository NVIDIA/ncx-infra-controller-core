/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//!
//! Carbide API module specific to measured boot/machine attestation.

pub mod metrics_collector;
pub mod rpc;

#[cfg(test)]
pub mod tests;
