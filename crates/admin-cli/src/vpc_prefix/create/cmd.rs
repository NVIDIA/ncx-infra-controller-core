/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::output::{FormattedOutput, OutputFormat};
use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};

use super::args::Args;
use crate::rpc::ApiClient;
use crate::vpc_prefix::show::cmd::ShowOutput;

pub async fn create(
    args: Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let output = api_client
        .0
        .create_vpc_prefix(args)
        .await
        .map(ShowOutput::One)?;

    output
        .write_output(output_format, ::rpc::admin_cli::Destination::Stdout())
        .map_err(CarbideCliError::from)
}
