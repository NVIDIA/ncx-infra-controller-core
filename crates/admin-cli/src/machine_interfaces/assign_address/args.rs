/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::IpAddr;

use carbide_uuid::machine::MachineInterfaceId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The machine interface ID to assign the address to")]
    pub interface_id: MachineInterfaceId,

    #[clap(help = "The IP address to assign (IPv4 or IPv6)")]
    pub ip_address: IpAddr,
}
