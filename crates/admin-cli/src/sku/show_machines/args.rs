/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use super::super::common::ShowSkuOptions;

// Args wraps the shared ShowSkuOptions as a subcommand
// specific newtype to allow sharing of ShowSkuOptions, and still
// providing a subcommand-specific Run trait implementation.
#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    pub inner: ShowSkuOptions,
}
