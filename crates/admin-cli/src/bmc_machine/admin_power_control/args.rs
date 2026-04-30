/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge as forgerpc;

use crate::bmc_machine::common::AdminPowerControlAction;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(long, help = "ID of the machine to reboot")]
    pub machine: String,
    #[clap(long, help = "Power control action")]
    pub action: AdminPowerControlAction,
}

impl From<Args> for forgerpc::AdminPowerControlRequest {
    fn from(args: Args) -> Self {
        Self {
            bmc_endpoint_request: None,
            machine_id: Some(args.machine),
            action: forgerpc::admin_power_control_request::SystemPowerControl::from(args.action)
                .into(),
        }
    }
}
