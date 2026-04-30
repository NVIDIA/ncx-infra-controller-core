/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "Tenant org ID to update", default_value(None))]
    pub tenant_org: String,

    #[clap(
        short = 'p',
        long,
        help = "Optional, routing profile name to apply to the tenant",
        default_value(None)
    )]
    pub routing_profile_type: Option<String>,

    #[clap(
        short = 'v',
        long,
        help = "Optional, version to use for comparison when performing the update, which will be rejected if the actual version of the record does not match the value of this parameter"
    )]
    pub version: Option<String>,

    #[clap(short = 'n', long, help = "Organization name of the tenant")]
    pub name: Option<String>,
}
