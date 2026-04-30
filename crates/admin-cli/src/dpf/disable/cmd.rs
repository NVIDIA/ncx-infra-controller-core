/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// Disable shares the same modify_dpf_state logic as enable,
// re-exported from the enable subcommand module.
pub use crate::dpf::enable::cmd::modify_dpf_state;
