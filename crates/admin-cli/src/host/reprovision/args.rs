/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge::host_reprovisioning_request::Mode;
use ::rpc::forge::{HostReprovisioningRequest, UpdateInitiator};
use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub enum Args {
    #[clap(about = "Set the host in reprovisioning mode.")]
    Set(ReprovisionSet),
    #[clap(about = "Clear the reprovisioning mode.")]
    Clear(ReprovisionClear),
    #[clap(about = "List all hosts pending reprovisioning.")]
    List,
    // TODO: Remove when manual upgrade feature is removed
    #[clap(about = "Mark manual firmware upgrade as complete for a host.")]
    MarkManualUpgradeComplete(ManualFirmwareUpgradeComplete),
}

#[derive(Parser, Debug, Clone)]
pub struct ReprovisionSet {
    #[clap(short, long, help = "Machine ID for which reprovisioning is needed.")]
    pub id: MachineId,

    #[clap(short, long, action)]
    pub update_firmware: bool,

    #[clap(
        long,
        alias = "maintenance_reference",
        help = "If set, a HostUpdateInProgress health alert will be applied to the host"
    )]
    pub update_message: Option<String>,
}

impl From<&ReprovisionSet> for HostReprovisioningRequest {
    fn from(args: &ReprovisionSet) -> Self {
        Self {
            machine_id: Some(args.id),
            mode: Mode::Set as i32,
            initiator: UpdateInitiator::AdminCli as i32,
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub struct ReprovisionClear {
    #[clap(
        short,
        long,
        help = "Machine ID for which reprovisioning should be cleared."
    )]
    pub id: MachineId,

    #[clap(short, long, action)]
    pub update_firmware: bool,
}

impl From<ReprovisionClear> for HostReprovisioningRequest {
    fn from(args: ReprovisionClear) -> Self {
        Self {
            machine_id: Some(args.id),
            mode: Mode::Clear as i32,
            initiator: UpdateInitiator::AdminCli as i32,
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub struct ManualFirmwareUpgradeComplete {
    #[clap(
        short,
        long,
        help = "Machine ID for which manual firmware upgrade should be set."
    )]
    pub id: MachineId,
}
