/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use forge_ssh::ssh::enable_rshim;

use super::super::common::SshArgs;

pub async fn enable_rshim_cmd(args: SshArgs) -> CarbideCliResult<()> {
    enable_rshim(
        args.credentials.bmc_ip_address,
        args.credentials.bmc_username,
        args.credentials.bmc_password,
    )
    .await
    .map_err(|e| CarbideCliError::GenericError(e.to_string()))?;
    Ok(())
}
