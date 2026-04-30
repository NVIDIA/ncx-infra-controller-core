/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use rpc::{CredentialType, forge as forgerpc};

use crate::credential::common::password_validator;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, required(true), help = "Leaf BGP session password")]
    pub password: String,
}

impl TryFrom<Args> for forgerpc::CredentialCreationRequest {
    type Error = CarbideCliError;

    fn try_from(args: Args) -> CarbideCliResult<Self> {
        Ok(Self {
            credential_type: CredentialType::BgpSiteWideLeafPassword.into(),
            username: None,
            password: password_validator(args.password)?,
            mac_address: None,
            vendor: None,
        })
    }
}
