/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use super::super::common::SshArgs;

// GetRshimStatus wraps the shared SshArgs as a subcommand
// specific newtype to allow sharing of SshArgs, and still
// providing a subcommand-specific Run trait implementation.
#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(flatten)]
    pub inner: SshArgs,
}
