/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use super::super::common::ExploreOptions;

// Args wraps the shared ExploreOptions as a subcommand
// specific newtype to allow sharing of ExploreOptions, and still
// providing a subcommand-specific Run trait implementation.
#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    pub inner: ExploreOptions,
}
