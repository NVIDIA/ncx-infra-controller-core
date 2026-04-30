/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use mac_address::MacAddress;

#[derive(Parser, Debug)]
pub struct ExploreOptions {
    #[clap(help = "BMC IP address or hostname with optional port")]
    pub address: String,
    #[clap(long, help = "The MAC address the BMC sent DHCP from")]
    pub mac: Option<MacAddress>,
}
