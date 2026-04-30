/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use prettytable::{Table, row};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn list(data: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let response = api_client.0.admin_list_resource_pools(data).await?;
    if response.pools.is_empty() {
        println!("No resource pools defined");
        return Err(CarbideCliError::Empty);
    }

    let mut table = Table::new();
    table.set_titles(row!["Name", "Min", "Max", "Size", "Used"]);
    for pool in response.pools {
        table.add_row(row![
            pool.name,
            pool.min,
            pool.max,
            pool.total,
            format!(
                "{} ({:.0}%)",
                pool.allocated,
                pool.allocated as f64 / pool.total as f64 * 100.0
            ),
        ]);
    }
    table.printstd();
    Ok(())
}
