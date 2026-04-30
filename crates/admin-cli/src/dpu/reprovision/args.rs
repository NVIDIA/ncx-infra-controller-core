/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;
use rpc::forge::dpu_reprovisioning_request::Mode;
use rpc::forge::{DpuReprovisioningRequest, UpdateInitiator};

#[derive(Parser, Debug)]
pub enum Args {
    #[clap(about = "Set the DPU in reprovisioning mode.")]
    Set(DpuReprovisionSet),
    #[clap(about = "Clear the reprovisioning mode.")]
    Clear(DpuReprovisionClear),
    #[clap(about = "List all DPUs pending reprovisioning.")]
    List,
    #[clap(about = "Restart the DPU reprovision.")]
    Restart(DpuReprovisionRestart),
}

#[derive(Parser, Debug)]
pub struct DpuReprovisionSet {
    #[clap(
        short,
        long,
        help = "DPU Machine ID for which reprovisioning is needed, or host machine id if all DPUs should be reprovisioned."
    )]
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

impl From<&DpuReprovisionSet> for DpuReprovisioningRequest {
    fn from(args: &DpuReprovisionSet) -> Self {
        Self {
            dpu_id: Some(args.id),
            machine_id: Some(args.id),
            mode: Mode::Set as i32,
            initiator: UpdateInitiator::AdminCli as i32,
            update_firmware: args.update_firmware,
        }
    }
}

#[derive(Parser, Debug)]
pub struct DpuReprovisionClear {
    #[clap(
        short,
        long,
        help = "DPU Machine ID for which reprovisioning should be cleared, or host machine id if all DPUs should be cleared."
    )]
    pub id: MachineId,

    #[clap(short, long, action)]
    pub update_firmware: bool,
}

impl From<&DpuReprovisionClear> for DpuReprovisioningRequest {
    fn from(args: &DpuReprovisionClear) -> Self {
        Self {
            dpu_id: Some(args.id),
            machine_id: Some(args.id),
            mode: Mode::Clear as i32,
            initiator: UpdateInitiator::AdminCli as i32,
            update_firmware: args.update_firmware,
        }
    }
}

#[derive(Parser, Debug)]
pub struct DpuReprovisionRestart {
    #[clap(
        short,
        long,
        help = "Host Machine ID for which reprovisioning should be restarted."
    )]
    pub id: MachineId,

    #[clap(short, long, action)]
    pub update_firmware: bool,
}

impl From<&DpuReprovisionRestart> for DpuReprovisioningRequest {
    fn from(args: &DpuReprovisionRestart) -> Self {
        Self {
            dpu_id: Some(args.id),
            machine_id: Some(args.id),
            mode: Mode::Restart as i32,
            initiator: UpdateInitiator::AdminCli as i32,
            update_firmware: args.update_firmware,
        }
    }
}
