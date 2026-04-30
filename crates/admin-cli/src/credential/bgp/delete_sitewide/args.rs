/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::{CredentialType, forge as forgerpc};

#[derive(Parser, Debug, Clone)]
pub struct Args {}

impl From<Args> for forgerpc::CredentialDeletionRequest {
    fn from(_: Args) -> Self {
        Self {
            credential_type: CredentialType::BgpSiteWideLeafPassword.into(),
            username: None,
            mac_address: None,
        }
    }
}
