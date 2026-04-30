/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use ::rpc::forge::CreateComputeAllocationRequest;

use super::args::Args;
use crate::compute_allocation::common::convert_compute_allocations_to_table;
use crate::rpc::ApiClient;

/// Create a compute allocation.
/// On successful creation, the details of the
/// new allocation will be displayed.
pub async fn create(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let req: CreateComputeAllocationRequest = args.try_into()?;
    let allocation = api_client.0.create_compute_allocation(req).await?;
    let allocation = allocation.allocation.ok_or(CarbideCliError::Empty)?;

    match output_format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&allocation).map_err(CarbideCliError::JsonError)?
        ),
        OutputFormat::Yaml => println!(
            "{}",
            serde_yaml::to_string(&allocation).map_err(CarbideCliError::YamlError)?
        ),
        OutputFormat::Csv => {
            convert_compute_allocations_to_table(vec![allocation], true)?
                .to_csv(std::io::stdout())
                .map_err(CarbideCliError::CsvError)?
                .flush()?;
        }
        _ => convert_compute_allocations_to_table(vec![allocation], true)?.printstd(),
    }

    Ok(())
}
