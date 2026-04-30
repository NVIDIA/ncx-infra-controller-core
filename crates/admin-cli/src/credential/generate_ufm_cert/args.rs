/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::{CredentialType, forge as forgerpc};

use crate::credential::common::DEFAULT_IB_FABRIC_NAME;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, default_value_t = DEFAULT_IB_FABRIC_NAME.to_string(), help = "Infiniband fabric.")]
    pub fabric: String,
}

impl From<Args> for forgerpc::CredentialCreationRequest {
    fn from(args: Args) -> Self {
        Self {
            credential_type: CredentialType::Ufm.into(),
            username: None,
            password: "".to_string(),
            mac_address: None,
            vendor: Some(args.fabric),
        }
    }
}
