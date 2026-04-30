/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use mac_address::MacAddress;
use rpc::{CredentialType, forge as forgerpc};

use crate::credential::common::BmcCredentialType;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(
        long,
        require_equals(true),
        required(true),
        help = "The BMC Credential kind"
    )]
    pub kind: BmcCredentialType,
    #[clap(long, help = "The MAC address of the BMC")]
    pub mac_address: Option<MacAddress>,
}

impl From<Args> for forgerpc::CredentialDeletionRequest {
    fn from(args: Args) -> Self {
        Self {
            credential_type: CredentialType::from(args.kind).into(),
            username: None,
            mac_address: args.mac_address.map(|mac| mac.to_string()),
        }
    }
}
