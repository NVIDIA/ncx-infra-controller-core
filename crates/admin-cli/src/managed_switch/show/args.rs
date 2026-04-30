/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::str::FromStr;

use carbide_uuid::switch::SwitchId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Switch ID or name to show details for (leave empty for all)")]
    pub identifier: Option<String>,

    #[clap(
        short,
        long,
        action,
        help = "Show BMC/NVOS MAC details in summary",
        conflicts_with = "identifier"
    )]
    pub ips: bool,

    #[clap(
        short,
        long,
        action,
        help = "Show serial, power, and health details in summary",
        conflicts_with = "identifier"
    )]
    pub more: bool,
}

impl Args {
    pub fn parse_identifier(&self) -> (Option<SwitchId>, Option<String>) {
        match &self.identifier {
            Some(id) if !id.is_empty() => match SwitchId::from_str(id) {
                Ok(switch_id) => (Some(switch_id), None),
                Err(_) => (None, Some(id.clone())),
            },
            _ => (None, None),
        }
    }
}
