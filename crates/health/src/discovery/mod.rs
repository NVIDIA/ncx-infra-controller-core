/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod cleanup;
mod context;
mod iteration;
mod spawn;

pub(crate) use context::BmcClient;
pub use context::DiscoveryLoopContext;
pub use iteration::run_discovery_iteration;

#[derive(Debug, Clone)]
pub struct DiscoveryIterationStats {
    pub discovered_endpoints: usize,
    pub sharded_endpoints: usize,
    pub active_monitors: usize,
}
