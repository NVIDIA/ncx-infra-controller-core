/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::cfg::cli_options::SortField;

/// Global options passed to instance commands
pub struct GlobalOptions<'a> {
    pub format: rpc::admin_cli::OutputFormat,
    pub page_size: usize,
    pub sort_by: &'a SortField,
    pub cloud_unsafe_op: Option<String>,
}
