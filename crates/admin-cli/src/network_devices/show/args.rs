/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        short,
        long,
        action,
        conflicts_with = "id",
        help = "Show all network devices (DEPRECATED)"
    )]
    pub all: bool,

    #[clap(
        default_value(""),
        help = "Show data for the given network device (e.g. `mac=<mac>`), leave empty for all (default)"
    )]
    pub id: String,
}

impl From<Args> for ::rpc::forge::NetworkTopologyRequest {
    fn from(args: Args) -> Self {
        let id = if args.all || args.id.is_empty() {
            None
        } else {
            Some(args.id)
        };
        Self { id }
    }
}
