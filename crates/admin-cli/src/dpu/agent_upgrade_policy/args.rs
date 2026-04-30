/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::{Parser, ValueEnum};
use rpc::forge::DpuAgentUpgradePolicyRequest;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long)]
    pub set: Option<AgentUpgradePolicyChoice>,
}

impl From<Args> for DpuAgentUpgradePolicyRequest {
    fn from(args: Args) -> Self {
        Self {
            new_policy: args.set.map(|choice| match choice {
                AgentUpgradePolicyChoice::Off => rpc::forge::AgentUpgradePolicy::Off as i32,
                AgentUpgradePolicyChoice::UpOnly => rpc::forge::AgentUpgradePolicy::UpOnly as i32,
                AgentUpgradePolicyChoice::UpDown => rpc::forge::AgentUpgradePolicy::UpDown as i32,
            }),
        }
    }
}

// Should match api/src/model/machine/upgrade_policy.rs AgentUpgradePolicy
#[derive(ValueEnum, Debug, Clone)]
pub enum AgentUpgradePolicyChoice {
    Off,
    UpOnly,
    UpDown,
}

impl std::fmt::Display for AgentUpgradePolicyChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // enums are a special case where their debug impl is their name ("Off")
        std::fmt::Debug::fmt(self, f)
    }
}

// From the RPC
impl From<i32> for AgentUpgradePolicyChoice {
    fn from(rpc_policy: i32) -> Self {
        use rpc::forge::AgentUpgradePolicy::*;
        match rpc_policy {
            n if n == Off as i32 => AgentUpgradePolicyChoice::Off,
            n if n == UpOnly as i32 => AgentUpgradePolicyChoice::UpOnly,
            n if n == UpDown as i32 => AgentUpgradePolicyChoice::UpDown,
            _ => {
                unreachable!();
            }
        }
    }
}
