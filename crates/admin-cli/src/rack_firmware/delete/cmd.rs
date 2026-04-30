/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete(opts: Args, api_client: &ApiClient) -> Result<(), CarbideCliError> {
    let id = opts.id.clone();

    match api_client.0.delete_rack_firmware(opts).await {
        Ok(_) => {
            println!("Deleted Rack firmware configuration: {}", id);
        }
        Err(status) if status.code() == tonic::Code::NotFound => {
            return Err(CarbideCliError::GenericError(format!(
                "Rack firmware configuration not found: {}",
                id
            )));
        }
        Err(err) => return Err(CarbideCliError::from(err)),
    }

    Ok(())
}
