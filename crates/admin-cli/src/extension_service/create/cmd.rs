/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::admin_cli::output::OutputFormat;

use super::super::show::cmd::convert_extension_services_to_table;
use super::args::Args;
use crate::rpc::ApiClient;

pub async fn handle_create(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let req: ::rpc::forge::CreateDpuExtensionServiceRequest = args.try_into()?;
    let extension_service = api_client.0.create_dpu_extension_service(req).await?;

    if is_json {
        println!("{}", serde_json::to_string_pretty(&extension_service)?);
    } else {
        convert_extension_services_to_table(&[extension_service]).printstd();
    }

    Ok(())
}
