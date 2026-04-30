/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Args {
    #[clap(about = "Show existing NVLink info")]
    Show(NvlinkInfoArgs),
    #[clap(about = "Build NVLink info from Redfish + NMX-M and populate DB")]
    Populate(NvlinkInfoPopulateArgs),
}

#[derive(Parser, Debug)]
pub struct NvlinkInfoArgs {
    #[clap(help = "Machine ID to query")]
    pub machine_id: MachineId,
}

#[derive(Parser, Debug)]
pub struct NvlinkInfoPopulateArgs {
    #[clap(help = "Machine ID to populate")]
    pub machine_id: MachineId,

    #[clap(long, action, help = "Update the database with the nvlink_info")]
    pub update_db: bool,
}
