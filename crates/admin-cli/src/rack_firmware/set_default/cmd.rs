/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliError;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn set_default(opts: Args, api_client: &ApiClient) -> Result<(), CarbideCliError> {
    let firmware_id = opts.firmware_id.clone();
    api_client.0.rack_firmware_set_default(opts).await?;
    println!("Set firmware '{}' as default.", firmware_id);
    Ok(())
}
