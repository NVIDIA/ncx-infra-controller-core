/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliResult, OutputFormat};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn network(
    api_client: &ApiClient,
    cmd: Args,
    format: OutputFormat,
    output_file: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
) -> CarbideCliResult<()> {
    match cmd {
        Args::Status => {
            println!(
                "Deprecated: Use dpu network, instead machine network. machine network will be removed in future."
            );
            crate::dpu::show_dpu_status(api_client, output_file).await?;
        }
        Args::Config(query) => {
            println!(
                "Deprecated: Use dpu network, instead of machine network. machine network will be removed in future."
            );
            let network_config = api_client
                .0
                .get_managed_host_network_config(query.machine_id)
                .await?;
            if format == OutputFormat::Json {
                println!("{}", serde_json::ser::to_string_pretty(&network_config)?);
            } else {
                // someone might be parsing this output
                println!("{network_config:?}");
            }
        }
    }
    Ok(())
}
