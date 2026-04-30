/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub use crate::health::HealthReportSources;

pub const HARDWARE_HEALTH_OVERRIDE_PREFIX: &str = "hardware-health.";

pub struct MaintenanceOverride {
    pub maintenance_reference: String,
    pub maintenance_start_time: Option<rpc::Timestamp>,
}

/// Machine-specific methods for HealthReportSources.
impl HealthReportSources {
    /// Derive legacy Maintenance mode fields.
    /// Determined by the value of a well-known health source, that is also set
    /// via SetMaintenance API.
    pub fn maintenance_override(&self) -> Option<MaintenanceOverride> {
        let ovr = self.merges.get("maintenance")?;
        let maintenance_alert_id = "Maintenance".parse().unwrap();
        let alert = ovr
            .alerts
            .iter()
            .find(|alert| alert.id == maintenance_alert_id)?;
        Some(MaintenanceOverride {
            maintenance_reference: alert.message.clone(),
            maintenance_start_time: alert.in_alert_since.map(rpc::Timestamp::from),
        })
    }

    pub fn is_hardware_health_override_source(source: &str) -> bool {
        source.starts_with(HARDWARE_HEALTH_OVERRIDE_PREFIX)
    }
}
