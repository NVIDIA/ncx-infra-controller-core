/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::str::FromStr;

use carbide_uuid::power_shelf::PowerShelfId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Power Shelf ID to delete.")]
    pub power_shelf_id: String,
}

impl Args {
    pub fn parse_power_shelf_id(&self) -> Result<PowerShelfId, String> {
        PowerShelfId::from_str(&self.power_shelf_id)
            .map_err(|_| format!("Invalid power shelf ID: {}", self.power_shelf_id))
    }
}
