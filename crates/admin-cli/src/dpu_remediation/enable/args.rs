/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::dpu_remediations::RemediationId;
use clap::Parser;
use rpc::forge::EnableRemediationRequest;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The id of the remediation to enable", long)]
    pub id: RemediationId,
}

impl From<Args> for EnableRemediationRequest {
    fn from(args: Args) -> Self {
        Self {
            remediation_id: Some(args.id),
        }
    }
}
