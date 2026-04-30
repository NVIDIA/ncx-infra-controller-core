/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use prettytable::{Table, row};

use super::args::Args;
use crate::async_write;
use crate::rpc::ApiClient;

pub async fn show(
    _args: &Args,
    format: OutputFormat,
    output_file: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let resp = api_client.0.list_host_firmware().await?;
    match format {
        OutputFormat::AsciiTable => {
            let mut table = Box::new(Table::new());
            table.set_titles(row![
                "Vendor",
                "Model",
                "Type",
                "Inventory Name",
                "Version",
                "Needs Explicit Start"
            ]);
            for row in resp.available {
                table.add_row(row![
                    row.vendor,
                    row.model,
                    row.r#type,
                    row.inventory_name_regex,
                    row.version,
                    row.needs_explicit_start,
                ]);
            }
            async_write!(output_file, "{}", table)?;
        }
        _ => {
            return Err(CarbideCliError::NotImplemented(
                "Format option not implemented".to_string(),
            ));
        }
    }
    Ok(())
}
