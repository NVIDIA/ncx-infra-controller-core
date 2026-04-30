/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use super::super::common::ExtensionServiceType;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        short = 'i',
        long,
        help = "The extension service ID to show (leave empty to show all)"
    )]
    pub id: Option<String>,

    #[clap(short = 't', long = "type", help = "Filter by service type (optional)")]
    pub service_type: Option<ExtensionServiceType>,

    #[clap(short = 'n', long = "name", help = "Filter by service name (optional)")]
    pub service_name: Option<String>,

    #[clap(
        short = 'o',
        long,
        help = "Filter by tenant organization ID (optional)"
    )]
    pub tenant_organization_id: Option<String>,
}
