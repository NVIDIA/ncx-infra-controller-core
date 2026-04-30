/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;
use std::collections::HashSet;

use super::context::{CollectorKind, DiscoveryLoopContext};

fn stop_collectors_for_keys(
    ctx: &mut DiscoveryLoopContext,
    kind: CollectorKind,
    removed_keys: &HashSet<Cow<'static, str>>,
) {
    let collectors = ctx.collectors.map_mut(kind);
    for key in removed_keys {
        if let Some(collector) = collectors.remove(key) {
            tracing::info!(
                endpoint_key = %key,
                remaining_collectors = collectors.len(),
                "{}",
                kind.stop_message()
            );
            tokio::spawn(async move {
                collector.stop().await;
            });
        }
    }
}

pub(super) fn stop_removed_bmc_collectors(
    ctx: &mut DiscoveryLoopContext,
    active_endpoints: &HashSet<Cow<'static, str>>,
) {
    let removed_keys = ctx.collectors.removed_keys(active_endpoints);
    for kind in CollectorKind::ALL {
        stop_collectors_for_keys(ctx, kind, &removed_keys);
    }

    if !removed_keys.is_empty() {
        tracing::info!(
            removed_count = removed_keys.len(),
            remaining_sensors = ctx.collectors.len(CollectorKind::Sensor),
            remaining_collectors = ctx.collectors.len(CollectorKind::Logs),
            remaining_firmware_collectors = ctx.collectors.len(CollectorKind::Firmware),
            remaining_nmxt_collectors = ctx.collectors.len(CollectorKind::Nmxt),
            remaining_nvue_rest_collectors = ctx.collectors.len(CollectorKind::NvueRest),
            "Cleaned up removed endpoints"
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_removed_keys_union_logic() {
        let mut maps = HashMap::new();
        maps.insert(
            CollectorKind::Sensor,
            HashMap::from([("a".to_string(), 1), ("b".to_string(), 2)]),
        );
        maps.insert(
            CollectorKind::Logs,
            HashMap::from([("b".to_string(), 3), ("c".to_string(), 4)]),
        );
        maps.insert(CollectorKind::Firmware, HashMap::new());
        maps.insert(CollectorKind::Nmxt, HashMap::new());
        maps.insert(CollectorKind::NvueRest, HashMap::new());

        let active = HashSet::from(["b".to_string()]);

        let removed: HashSet<String> = maps
            .values()
            .flat_map(|map| map.keys())
            .filter(|key| !active.contains(*key))
            .cloned()
            .collect();

        assert_eq!(removed, HashSet::from(["a".to_string(), "c".to_string()]));
    }
}
