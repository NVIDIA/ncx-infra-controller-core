/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use forge_ssh::ssh::read_obmc_console_log;

use super::super::common::SshArgs;

pub async fn show_obmc_log(args: SshArgs) -> CarbideCliResult<()> {
    let log = read_obmc_console_log(
        args.credentials.bmc_ip_address,
        args.credentials.bmc_username,
        args.credentials.bmc_password,
    )
    .await
    .map_err(|e| CarbideCliError::GenericError(e.to_string()))?;

    println!("OBMC Console Log:\n{log}");
    Ok(())
}
