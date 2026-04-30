/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use rpc::forge::DeleteOsImageRequest;

use crate::os_image::common::str_to_rpc_uuid;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(short = 'i', long, help = "uuid of the OS image to delete.")]
    pub id: String,
    #[clap(
        short = 't',
        long,
        help = "Tenant organization identifier of OS image to delete."
    )]
    pub tenant_org_id: String,
}

impl TryFrom<Args> for DeleteOsImageRequest {
    type Error = CarbideCliError;

    fn try_from(args: Args) -> CarbideCliResult<Self> {
        let id = str_to_rpc_uuid(&args.id)?;
        Ok(DeleteOsImageRequest {
            id: Some(id),
            tenant_organization_id: args.tenant_org_id,
        })
    }
}
