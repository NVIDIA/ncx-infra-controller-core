/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge::IpxeTemplateParameter;
use clap::Parser;

use crate::operating_system::common::parse_param;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "UUID of the operating system definition to update.")]
    pub id: String,

    #[clap(short, long, help = "New name for the operating system definition.")]
    pub name: Option<String>,

    #[clap(short, long, help = "New description.")]
    pub description: Option<String>,

    #[clap(long, help = "Set whether this OS definition is active.")]
    pub is_active: Option<bool>,

    #[clap(long, help = "Set whether users can override OS parameters.")]
    pub allow_override: Option<bool>,

    #[clap(long, help = "Set whether phone-home on first boot is enabled.")]
    pub phone_home_enabled: Option<bool>,

    #[clap(long, help = "Update the cloud-init / user-data script.")]
    pub user_data: Option<String>,

    #[clap(
        long,
        conflicts_with = "ipxe_template_id",
        help = "Update the raw iPXE boot script."
    )]
    pub ipxe_script: Option<String>,

    #[clap(
        long,
        conflicts_with = "ipxe_script",
        help = "Update the iPXE template ID."
    )]
    pub ipxe_template_id: Option<String>,

    #[clap(
        long = "param",
        value_name = "KEY=VALUE",
        value_parser = parse_param,
        num_args = 0..,
        help = "Replace all iPXE parameters with these KEY=VALUE pairs. May be repeated. Pass without values to clear."
    )]
    pub params: Option<Vec<IpxeTemplateParameter>>,
}
