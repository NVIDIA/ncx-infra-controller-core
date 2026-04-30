/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::{CredentialType, forge as forgerpc};

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, required(true), help = "Default username: root, ADMIN, etc")]
    pub username: String,
    #[clap(long, required(true), help = "DPU manufacturer default password")]
    pub password: String,
}

impl From<Args> for forgerpc::CredentialCreationRequest {
    fn from(args: Args) -> Self {
        Self {
            credential_type: CredentialType::DpuBmcFactoryDefault.into(),
            username: Some(args.username),
            password: args.password,
            mac_address: None,
            vendor: None,
        }
    }
}
