/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;
use rpc::forge as forgerpc;

/// Enable or disable quarantine mode on a managed host.
#[derive(Parser, Debug)]
pub enum Args {
    /// Put this machine into quarantine. Prevents any network access on the host machine.
    On(QuarantineOn),
    /// Take this machine out of quarantine
    Off(QuarantineOff),
}

#[derive(Parser, Debug)]
pub struct QuarantineOn {
    #[clap(long, required(true), help = "Managed Host ID")]
    pub host: MachineId,

    #[clap(
        long,
        visible_alias = "reason",
        required(true),
        help = "Reason for quarantining this host"
    )]
    pub reason: String,
}

impl From<QuarantineOn> for forgerpc::SetManagedHostQuarantineStateRequest {
    fn from(args: QuarantineOn) -> Self {
        Self {
            machine_id: Some(args.host),
            quarantine_state: Some(forgerpc::ManagedHostQuarantineState {
                mode: forgerpc::ManagedHostQuarantineMode::BlockAllTraffic as i32,
                reason: Some(args.reason),
            }),
        }
    }
}

#[derive(Parser, Debug)]
pub struct QuarantineOff {
    #[clap(long, required(true), help = "Managed Host ID")]
    pub host: MachineId,
}

impl From<QuarantineOff> for forgerpc::ClearManagedHostQuarantineStateRequest {
    fn from(args: QuarantineOff) -> Self {
        Self {
            machine_id: Some(args.host),
        }
    }
}
