/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;
use rpc::forge as forgerpc;

/// Enable or disable maintenance mode on a managed host.
/// To list machines in maintenance mode use `forge-admin-cli mh show --all --fix`
#[derive(Parser, Debug)]
pub enum Args {
    /// Put this machine into maintenance mode. Prevents an instance being assigned to it.
    On(MaintenanceOn),
    /// Return this machine to normal operation.
    Off(MaintenanceOff),
}

#[derive(Parser, Debug)]
pub struct MaintenanceOn {
    #[clap(long, required(true), help = "Managed Host ID")]
    pub host: MachineId,

    #[clap(
        long,
        visible_alias = "ref",
        required(true),
        help = "URL of reference (ticket, issue, etc) for this machine's maintenance"
    )]
    pub reference: String,
}

impl From<MaintenanceOn> for forgerpc::MaintenanceRequest {
    fn from(args: MaintenanceOn) -> Self {
        Self {
            operation: forgerpc::MaintenanceOperation::Enable.into(),
            host_id: Some(args.host),
            reference: Some(args.reference),
        }
    }
}

#[derive(Parser, Debug)]
pub struct MaintenanceOff {
    #[clap(long, required(true), help = "Managed Host ID")]
    pub host: MachineId,
}

impl From<MaintenanceOff> for forgerpc::MaintenanceRequest {
    fn from(args: MaintenanceOff) -> Self {
        Self {
            operation: forgerpc::MaintenanceOperation::Disable.into(),
            host_id: Some(args.host),
            reference: None,
        }
    }
}
