/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge as forgerpc;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn maintenance(api_client: &ApiClient, action: Args) -> CarbideCliResult<()> {
    let req: forgerpc::MaintenanceRequest = match action {
        Args::On(args) => args.into(),
        Args::Off(args) => args.into(),
    };
    api_client.0.set_maintenance(req).await?;
    Ok(())
}
