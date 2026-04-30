/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge::ClearHostUefiPasswordRequest;
use clap::Parser;

use crate::machine::MachineQuery;

// Args wraps the shared MachineQuery as a subcommand
// specific newtype to allow sharing of MachineQuery, and still
// providing a subcommand-specific Run trait implementation.
#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(flatten)]
    pub inner: MachineQuery,
}

impl From<Args> for ClearHostUefiPasswordRequest {
    fn from(args: Args) -> Self {
        Self {
            host_id: None,
            machine_query: Some(args.inner.query),
        }
    }
}
