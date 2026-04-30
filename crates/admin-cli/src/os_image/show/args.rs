/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::admin_cli::{CarbideCliError, CarbideCliResult};

use crate::os_image::common::str_to_rpc_uuid;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "uuid of the OS image to show.")]
    pub id: Option<String>,
    #[clap(
        short = 't',
        long,
        help = "Tenant organization identifier to filter OS images listing."
    )]
    pub tenant_org_id: Option<String>,
}

/// Represents the parsed query for the show command.
pub enum ShowQuery {
    /// Show a single OS image by its UUID.
    Single(::rpc::common::Uuid),
    /// List OS images, optionally filtered by tenant organization ID.
    List(Option<String>),
}

impl TryFrom<Args> for ShowQuery {
    type Error = CarbideCliError;

    fn try_from(args: Args) -> CarbideCliResult<Self> {
        match args.id {
            Some(id) => {
                let uuid = str_to_rpc_uuid(&id)?;
                Ok(ShowQuery::Single(uuid))
            }
            None => Ok(ShowQuery::List(args.tenant_org_id)),
        }
    }
}
