/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;
use ::rpc::forge as forgerpc;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        default_value(""),
        help = "The Tenant KeySet ID in the format of <tenant_org_id>/<keyset_id> to query, leave empty for all (default)"
    )]
    pub id: String,

    #[clap(short, long, help = "The Tenant Org ID to query")]
    pub tenant_org_id: Option<String>,
}

impl TryFrom<&Args> for Option<forgerpc::TenantKeysetIdentifier> {
    type Error = CarbideCliError;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        if args.id.is_empty() {
            return Ok(None);
        }

        let split_id = args.id.split('/').collect::<Vec<&str>>();
        if split_id.len() != 2 {
            return Err(CarbideCliError::GenericError(
                "Invalid format for Tenant KeySet ID".to_string(),
            ));
        }

        Ok(Some(forgerpc::TenantKeysetIdentifier {
            organization_id: split_id[0].to_string(),
            keyset_id: split_id[1].to_string(),
        }))
    }
}
