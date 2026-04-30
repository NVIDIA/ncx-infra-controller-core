/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};

use super::args::{Args, ShowQuery};
use crate::rpc::ApiClient;

pub async fn show(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
    _page_size: usize,
) -> CarbideCliResult<()> {
    let query: ShowQuery = args.try_into()?;
    let is_json = output_format == OutputFormat::Json;
    let images = match query {
        ShowQuery::Single(id) => vec![api_client.0.get_os_image(id).await?],
        ShowQuery::List(tenant_org_id) => api_client.list_os_image(tenant_org_id).await?,
    };
    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&images).map_err(CarbideCliError::JsonError)?
        );
    } else {
        // todo: pretty print in table form
        println!("{images:?}");
    }
    Ok(())
}
