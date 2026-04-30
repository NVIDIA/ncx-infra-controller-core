/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::MachineBootOverride;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn set(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let req: MachineBootOverride = args.try_into()?;
    api_client.0.set_machine_boot_override(req).await?;
    Ok(())
}
