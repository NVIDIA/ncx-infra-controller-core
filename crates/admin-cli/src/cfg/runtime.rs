/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::admin_cli::OutputFormat;

use crate::cfg::cli_options::SortField;
use crate::rpc::ApiClient;

// RuntimeContext is context passed to all subcommand
// dispatch handlers. This is built at the beginning of
// runtime and then passed to the appropriate dispatcher.
pub struct RuntimeContext {
    pub api_client: ApiClient,
    pub config: RuntimeConfig,
    pub output_file: Box<dyn tokio::io::AsyncWrite + Unpin>,
}

// RuntimeConfig contains runtime configuration parameters extracted
// from CLI options. This should contain the entirety of any options
// that need to be leveraged by any downstream command handler.
pub struct RuntimeConfig {
    pub format: OutputFormat,
    pub page_size: usize,
    pub extended: bool,
    pub cloud_unsafe_op_enabled: bool,
    pub sort_by: SortField,
}
