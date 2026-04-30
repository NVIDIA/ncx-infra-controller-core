/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use arc_swap::ArcSwap;
use carbide_utils::HostPortPair;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use super::logging::level_filter::ActiveLevel;

#[derive(Clone)]
pub struct DynamicSettings {
    /// RUST_LOG level
    pub log_filter: Arc<ActiveLevel>,

    /// Whether site-explorer is enabled (running periodic explorations)
    pub site_explorer_enabled: Arc<AtomicBool>,

    /// Should site-explorer create machines
    pub create_machines: Arc<AtomicBool>,

    /// Use a proxy for talking to BMC's
    pub bmc_proxy: Arc<ArcSwap<Option<HostPortPair>>>,

    /// Whether log tracing should be enabled
    pub tracing_enabled: Arc<AtomicBool>,
}

/// How often to check if the log filter (RUST_LOG) needs resetting
pub(crate) const RESET_PERIOD: Duration = Duration::from_secs(15 * 60); // 1/4 hour

impl DynamicSettings {
    /// The background task that resets dynamic features to their startup values when the override expires
    pub(crate) fn start_reset_task(
        &self,
        join_set: &mut JoinSet<()>,
        period: Duration,
        cancel_token: CancellationToken,
    ) {
        let log_filter = self.log_filter.clone();
        join_set
            .build_task()
            .name("dynamic_feature_reset")
            .spawn(async move {
                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(period) => {}
                        _ = cancel_token.cancelled() => {
                            break;
                        }
                    }

                    if let Err(err) = log_filter.reset_if_expired() {
                        tracing::error!("Failed resetting log level: {err}");
                    }
                }
            })
            // Safety: spawn only fails if outside the tokio runtime.
            .expect("Could not spawn dynamic_feature_reset task");
    }
}
