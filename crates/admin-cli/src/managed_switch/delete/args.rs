/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::str::FromStr;

use carbide_uuid::switch::SwitchId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Switch ID to delete.")]
    pub switch_id: String,
}

impl Args {
    pub fn parse_switch_id(&self) -> Result<SwitchId, String> {
        SwitchId::from_str(&self.switch_id)
            .map_err(|_| format!("Invalid switch ID: {}", self.switch_id))
    }
}
