/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub const PATH: &str = "etc/frr/daemons";
const TMPL_FULL: &str = include_str!("../templates/daemons");
pub const RESTART_CMD: &str = "supervisorctl restart frr";

/// Generate /etc/frr/daemons. It has no templated parts.
pub fn build() -> String {
    TMPL_FULL.to_string()
}
