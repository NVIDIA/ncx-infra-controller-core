/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use mac_address::MacAddress;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "BMC IP address or hostname with optional port")]
    pub address: String,
    #[clap(long, help = "The MAC address the BMC sent DHCP from")]
    pub mac: Option<MacAddress>,
    #[clap(
        long,
        help = "Host BMC IP address. Required for the mandatory post-copy host power-cycle \
                that applies the new BFB image to the DPU."
    )]
    pub host_bmc_ip: String,
    #[clap(
        long,
        help = "Power-cycle the host before the BFB copy to release rshim control to the DPU BMC."
    )]
    pub pre_copy_powercycle: bool,
}
