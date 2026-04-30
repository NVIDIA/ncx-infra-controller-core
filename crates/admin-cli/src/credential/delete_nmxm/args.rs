/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::{CredentialType, forge as forgerpc};

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, required(true), help = "NmxM url")]
    pub username: String,
}

impl From<Args> for forgerpc::CredentialDeletionRequest {
    fn from(args: Args) -> Self {
        Self {
            credential_type: CredentialType::NmxM.into(),
            username: Some(args.username),
            mac_address: None,
        }
    }
}
