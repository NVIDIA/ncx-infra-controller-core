/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineInterfaceId;
use clap::Parser;

use crate::boot_override::common::BootOverride;

// Args wraps the shared BootOverride as a subcommand
// specific newtype to allow sharing of BootOverride, and still
// providing a subcommand-specific Run trait implementation.
#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(flatten)]
    pub inner: BootOverride,
}

impl From<Args> for MachineInterfaceId {
    fn from(args: Args) -> Self {
        args.inner.interface_id
    }
}
