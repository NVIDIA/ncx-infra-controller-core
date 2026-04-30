/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge::IpxeTemplateParameter;
use clap::Parser;

use crate::operating_system::common::parse_param;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short, long, help = "Name of the operating system definition.")]
    pub name: String,

    #[clap(short, long, help = "Organization identifier for this OS definition.")]
    pub org: String,

    #[clap(
        long,
        help = "Optional UUID for the new OS definition (default: server-generated)."
    )]
    pub id: Option<String>,

    #[clap(short, long, help = "Optional human-readable description.")]
    pub description: Option<String>,

    #[clap(long, help = "Whether this OS definition is active (default: true).")]
    pub is_active: Option<bool>,

    #[clap(
        long,
        default_value = "false",
        help = "Allow users to override OS parameters."
    )]
    pub allow_override: bool,

    #[clap(
        long,
        default_value = "false",
        help = "Enable phone-home on first boot."
    )]
    pub phone_home_enabled: bool,

    #[clap(long, help = "Optional cloud-init / user-data script.")]
    pub user_data: Option<String>,

    #[clap(
        long,
        conflicts_with_all = ["ipxe_template_id"],
        help = "Raw iPXE boot script (mutually exclusive with --ipxe-template-id)."
    )]
    pub ipxe_script: Option<String>,

    #[clap(
        long,
        conflicts_with_all = ["ipxe_script"],
        help = "ID of the iPXE template to use (mutually exclusive with --ipxe-script)."
    )]
    pub ipxe_template_id: Option<String>,

    #[clap(
        long = "param",
        value_name = "KEY=VALUE",
        value_parser = parse_param,
        help = "iPXE parameter in KEY=VALUE format. May be repeated."
    )]
    pub params: Vec<IpxeTemplateParameter>,
}
