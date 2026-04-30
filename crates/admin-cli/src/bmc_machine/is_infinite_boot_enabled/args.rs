/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge as forgerpc;

use crate::bmc_machine::common::InfiniteBootArgs;

// IsInfiniteBootEnabledArgs wraps the shared InfiniteBootArgs as a subcommand
// specific newtype to allow sharing of InfiniteBootArgs, and still
// providing a subcommand-specific Run trait implementation.
#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(flatten)]
    pub inner: InfiniteBootArgs,
}

impl From<Args> for forgerpc::IsInfiniteBootEnabledRequest {
    fn from(args: Args) -> Self {
        Self {
            bmc_endpoint_request: None,
            machine_id: Some(args.inner.machine),
        }
    }
}
