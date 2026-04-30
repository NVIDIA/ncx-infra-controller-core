/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod create;
mod delete;
mod show;

#[cfg(test)]
mod tests;

use ::rpc::admin_cli::CarbideCliResult;
use clap::Parser;
use prettytable::{Table, row};
use rpc::forge::VpcPeering;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Create VPC peering.")]
    Create(create::Args),
    #[clap(about = "Show list of VPC peerings.")]
    Show(show::Args),
    #[clap(about = "Delete VPC peering.")]
    Delete(delete::Args),
}

fn convert_vpc_peerings_to_table(vpc_peerings: &[VpcPeering]) -> CarbideCliResult<Box<Table>> {
    let mut table = Box::new(Table::new());

    table.set_titles(row!["Id", "VPC1 ID", "VPC2 ID"]);

    for vpc_peering in vpc_peerings {
        let id = vpc_peering.id.map(|id| id.to_string()).unwrap_or_default();
        let vpc_id = vpc_peering
            .vpc_id
            .as_ref()
            .map(|uuid| uuid.to_string())
            .unwrap_or("None".to_string());
        let peer_vpc_id = vpc_peering
            .peer_vpc_id
            .as_ref()
            .map(|uuid| uuid.to_string())
            .unwrap_or("None".to_string());

        table.add_row(row![id, vpc_id, peer_vpc_id]);
    }

    Ok(table)
}
