/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::{Parser, ValueEnum};
use rpc::forge::{self as forgerpc, PowerOptionUpdateRequest};

#[derive(Parser, Debug)]
pub enum Args {
    Show(ShowPowerOptions),
    Update(UpdatePowerOptions),
    #[clap(about = "Get machine ingestion state")]
    GetMachineIngestionState(BmcMacAddress),
    #[clap(about = "Allow a machine to power on")]
    AllowIngestionAndPowerOn(BmcMacAddress),
}

#[derive(Parser, Debug)]
pub struct ShowPowerOptions {
    #[clap(help = "ID of the host or nothing for all")]
    pub machine: Option<MachineId>,
}

#[derive(Parser, Debug)]
pub struct UpdatePowerOptions {
    #[clap(help = "ID of the host")]
    pub machine: MachineId,
    #[clap(long, short, help = "Desired Power State")]
    pub desired_power_state: DesiredPowerState,
}

impl From<UpdatePowerOptions> for PowerOptionUpdateRequest {
    fn from(args: UpdatePowerOptions) -> Self {
        let power_state = match args.desired_power_state {
            DesiredPowerState::On => forgerpc::PowerState::On,
            DesiredPowerState::Off => forgerpc::PowerState::Off,
            DesiredPowerState::PowerManagerDisabled => forgerpc::PowerState::PowerManagerDisabled,
        };
        Self {
            machine_id: Some(args.machine),
            power_state: power_state as i32,
        }
    }
}

#[derive(ValueEnum, Parser, Debug, Clone, PartialEq)]
pub enum DesiredPowerState {
    On,
    Off,
    PowerManagerDisabled,
}

#[derive(Parser, Debug)]
pub struct BmcMacAddress {
    #[clap(short, long, help = "MAC Address of host BMC endpoint")]
    pub mac_address: mac_address::MacAddress,
}
