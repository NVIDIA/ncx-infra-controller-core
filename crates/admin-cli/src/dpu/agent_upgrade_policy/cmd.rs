/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::{AgentUpgradePolicyChoice, Args};
use crate::rpc::ApiClient;

pub async fn agent_upgrade_policy(api_client: &ApiClient, args: Args) -> CarbideCliResult<()> {
    let is_set = args.set.is_some();
    let resp = api_client.0.dpu_agent_upgrade_policy_action(args).await?;
    let policy: AgentUpgradePolicyChoice = resp.active_policy.into();

    if is_set {
        tracing::info!(
            "Policy is now: {policy}. Update succeeded? {}.",
            resp.did_change
        );
    } else {
        tracing::info!("{policy}");
    }

    Ok(())
}
