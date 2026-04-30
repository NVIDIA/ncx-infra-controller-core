/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Machine ID")]
    pub machine_id: MachineId,
    #[clap(help = "MAC address (e.g. 00:11:22:33:44:55)")]
    pub mac_addr: String,
    #[clap(help = "Device type (e.g. BlueField3)")]
    pub device_type: String,
    #[clap(help = "PCI name (e.g. 5e:00.0)")]
    pub pci_name: String,
}

impl From<Args> for ::rpc::forge::DpaInterfaceCreationRequest {
    fn from(args: Args) -> Self {
        Self {
            machine_id: Some(args.machine_id),
            mac_addr: args.mac_addr,
            device_type: args.device_type,
            pci_name: args.pci_name,
        }
    }
}
