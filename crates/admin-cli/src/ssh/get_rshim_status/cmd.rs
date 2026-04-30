/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use forge_ssh::ssh::is_rshim_enabled;

use super::super::common::SshArgs;

pub async fn get_rshim_status(args: SshArgs) -> CarbideCliResult<()> {
    let is_rshim_enabled = is_rshim_enabled(
        args.credentials.bmc_ip_address,
        args.credentials.bmc_username,
        args.credentials.bmc_password,
    )
    .await
    .map_err(|e| CarbideCliError::GenericError(e.to_string()))?;
    tracing::info!("{is_rshim_enabled}");
    Ok(())
}
