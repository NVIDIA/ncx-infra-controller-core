/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliResult, OutputFormat};

use super::args::Args;
use crate::rpc::ApiClient;
use crate::sku::show::cmd::show_sku_details;

pub async fn generate(
    args: Args,
    api_client: &ApiClient,
    output: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
    output_format: &OutputFormat,
    extended: bool,
) -> CarbideCliResult<()> {
    let mut sku = api_client
        .0
        .generate_sku_from_machine(args.machine_id)
        .await?;
    if let Some(id) = args.id {
        sku.id = id;
    }
    show_sku_details(output, output_format, extended, sku).await?;
    Ok(())
}
