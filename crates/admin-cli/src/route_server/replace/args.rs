/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use crate::route_server::common::AddressArgs;

// Args wraps the shared AddressArgs as a subcommand
// specific newtype to allow sharing of AddressArgs, and still
// providing a subcommand-specific Run trait implementation.
#[derive(Parser, Debug)]
pub struct Args {
    #[clap(flatten)]
    pub inner: AddressArgs,
}
