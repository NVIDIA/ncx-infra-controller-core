/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliResult, OutputFormat};
use ::rpc::forge as forgerpc;
use prettytable::{Cell, Row, Table};
use serde::Serialize;

use super::args::Args;
use crate::rpc::ApiClient;

#[derive(Serialize)]
struct AddressRow {
    address: String,
    family: String,
    allocation_type: String,
}

pub async fn handle_show_addresses(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let resp = api_client
        .0
        .find_interface_addresses(forgerpc::FindInterfaceAddressesRequest {
            interface_id: Some(args.interface_id),
        })
        .await?;

    let rows: Vec<AddressRow> = resp
        .addresses
        .iter()
        .map(|a| AddressRow {
            address: a.address.clone(),
            family: if a.address.contains(':') {
                "IPv6".into()
            } else {
                "IPv4".into()
            },
            allocation_type: a.allocation_type.clone(),
        })
        .collect();

    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        _ => {
            if rows.is_empty() {
                println!("No addresses found for interface {}", args.interface_id);
            } else {
                let mut table = Table::new();
                table.set_titles(Row::new(vec![
                    Cell::new("Address"),
                    Cell::new("Family"),
                    Cell::new("Type"),
                ]));

                for row in &rows {
                    table.add_row(Row::new(vec![
                        Cell::new(&row.address),
                        Cell::new(&row.family),
                        Cell::new(&row.allocation_type),
                    ]));
                }

                table.printstd();
            }
        }
    }

    Ok(())
}
