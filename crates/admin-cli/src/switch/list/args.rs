/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use mac_address::MacAddress;
use rpc::forge::DeletedFilter;

#[derive(Parser, Debug)]
pub struct Args {
    /// Include deleted switches
    #[clap(long, value_enum, default_value = "exclude")]
    pub deleted: DeletedFilter,

    /// Filter by controller state (e.g. "ready", "initializing", "error")
    #[clap(long)]
    pub controller_state: Option<String>,

    /// Filter by BMC MAC address
    #[clap(long)]
    pub bmc_mac: Option<MacAddress>,

    /// Filter by NVOS MAC address
    #[clap(long)]
    pub nvos_mac: Option<MacAddress>,
}
