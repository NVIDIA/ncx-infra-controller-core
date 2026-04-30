/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::dpu_remediations::RemediationId;
use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(
        help = "The remediation id to query, in case the user wants to see which machines have a specific remediation applied.  Provide both arguments to see all the details for a specific remediation and machine.",
        long
    )]
    pub remediation_id: Option<RemediationId>,
    #[clap(
        help = "The machine id to query, in case the user wants to see which remediations have been applied to a specific box.  Provide both arguments to see all the details for a specific remediation and machine.",
        long
    )]
    pub machine_id: Option<MachineId>,
}
