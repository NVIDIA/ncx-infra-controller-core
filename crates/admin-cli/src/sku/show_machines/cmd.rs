/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use ::rpc::forge::SkuList;
use prettytable::{Row, Table};
use tokio::io::AsyncWriteExt;

use super::super::common::ShowSkuOptions;
use crate::rpc::ApiClient;

async fn show_machine_table(
    output_file: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
    output_format: &OutputFormat,
    skus: Vec<::rpc::forge::Sku>,
) -> CarbideCliResult<()> {
    if *output_format != OutputFormat::AsciiTable {
        return Err(CarbideCliError::GenericError(
            "Only ascii table format supported".to_string(),
        ));
    }

    let mut output = Vec::default();
    let mut table = Table::new();
    table.set_titles(Row::from(vec!["SKU ID", "Assigned Machine IDs"]));

    for sku in skus {
        let machines = sku
            .associated_machine_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        table.add_row(Row::from(vec![sku.id, machines]));
    }
    table.print(&mut output)?;
    output_file.write_all(output.as_slice()).await?;
    Ok(())
}

pub async fn show_machines(
    args: ShowSkuOptions,
    api_client: &ApiClient,
    output: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
    output_format: &OutputFormat,
) -> CarbideCliResult<()> {
    if let Some(sku_id) = args.sku_id {
        let skus = api_client.0.find_skus_by_ids(vec![sku_id]).await?;
        show_machine_table(output, output_format, skus.skus).await?;
    } else {
        let all_ids = api_client.0.get_all_sku_ids().await?;
        let sku_list = if !all_ids.ids.is_empty() {
            api_client.0.find_skus_by_ids(all_ids.ids).await?
        } else {
            SkuList::default()
        };

        show_machine_table(output, output_format, sku_list.skus).await?;
    };

    Ok(())
}
