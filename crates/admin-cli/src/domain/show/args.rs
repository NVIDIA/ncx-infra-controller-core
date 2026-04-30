/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::domain::DomainId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        short,
        long,
        action,
        conflicts_with = "domain",
        help = "Show all domains (DEPRECATED)"
    )]
    pub all: bool,

    #[clap(
        default_value(None),
        help = "The domain to query, leave empty for all (default)"
    )]
    pub domain: Option<DomainId>,
}
