/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::OnDemandOptions;
use crate::rpc::ApiClient;

pub async fn on_demand_machine_validation(
    api_client: &ApiClient,
    args: OnDemandOptions,
) -> CarbideCliResult<()> {
    api_client
        .on_demand_machine_validation(
            args.machine,
            args.tags,
            args.allowed_tests,
            args.run_unverfied_tests,
            args.contexts,
        )
        .await?;
    Ok(())
}
