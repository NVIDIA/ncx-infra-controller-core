/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use crate::component_manager::common::DeviceTargetArgs;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub target: DeviceTargetArgs,
}

impl From<Args> for rpc::forge::GetComponentFirmwareStatusRequest {
    fn from(args: Args) -> Self {
        match args.target {
            DeviceTargetArgs::Switch(target) => Self {
                target: Some(
                    rpc::forge::get_component_firmware_status_request::Target::SwitchIds(
                        target.into(),
                    ),
                ),
            },
            DeviceTargetArgs::PowerShelf(target) => Self {
                target: Some(
                    rpc::forge::get_component_firmware_status_request::Target::PowerShelfIds(
                        target.into(),
                    ),
                ),
            },
            DeviceTargetArgs::ComputeTray(target) => Self {
                target: Some(
                    rpc::forge::get_component_firmware_status_request::Target::MachineIds(
                        target.into(),
                    ),
                ),
            },
            DeviceTargetArgs::Rack(target) => Self {
                target: Some(
                    rpc::forge::get_component_firmware_status_request::Target::RackIds(
                        target.into(),
                    ),
                ),
            },
        }
    }
}
