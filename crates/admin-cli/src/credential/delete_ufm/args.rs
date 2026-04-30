/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use rpc::{CredentialType, forge as forgerpc};

use crate::credential::common::url_validator;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, required(true), help = "The UFM url")]
    pub url: String,
}

impl TryFrom<Args> for forgerpc::CredentialDeletionRequest {
    type Error = CarbideCliError;
    fn try_from(args: Args) -> CarbideCliResult<Self> {
        let username = url_validator(args.url)?;
        Ok(Self {
            credential_type: CredentialType::Ufm.into(),
            username: Some(username),
            mac_address: None,
        })
    }
}
