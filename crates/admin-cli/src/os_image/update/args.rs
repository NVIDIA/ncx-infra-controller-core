/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::admin_cli::{CarbideCliError, CarbideCliResult};

use crate::os_image::common::str_to_rpc_uuid;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "uuid of the OS image to update.")]
    pub id: String,
    #[clap(short = 'n', long, help = "Optional, name of the OS image entry.")]
    pub name: Option<String>,
    #[clap(
        short = 'd',
        long,
        help = "Optional, description of the OS image entry."
    )]
    pub description: Option<String>,
    #[clap(
        short = 'y',
        long,
        help = "Optional, Authentication type, usually Bearer."
    )]
    pub auth_type: Option<String>,
    #[clap(
        short = 'p',
        long,
        help = "Optional, Authentication token, usually in base64."
    )]
    pub auth_token: Option<String>,
}

/// Parsed update request with a validated UUID.
pub struct UpdateRequest {
    pub id: ::rpc::common::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub auth_type: Option<String>,
    pub auth_token: Option<String>,
}

impl TryFrom<Args> for UpdateRequest {
    type Error = CarbideCliError;

    fn try_from(args: Args) -> CarbideCliResult<Self> {
        let id = str_to_rpc_uuid(&args.id)?;
        Ok(UpdateRequest {
            id,
            name: args.name,
            description: args.description,
            auth_type: args.auth_type,
            auth_token: args.auth_token,
        })
    }
}
