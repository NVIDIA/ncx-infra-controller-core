/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::dpu_remediations::RemediationId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The remediation id to query, if not provided defaults to all")]
    pub id: Option<RemediationId>,
    #[clap(long, action)]
    pub display_script: bool,
}
