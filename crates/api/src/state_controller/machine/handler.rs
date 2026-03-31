/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! State Handler implementation for Machines
//!
//! This module serves as the dispatch hub for the machine state controller.
//! Each major state has its own submodule containing its handling logic.

use std::sync::Arc;

use carbide_uuid::machine::MachineId;
use chrono::{Duration, Utc};
use eyre::eyre;
use forge_secrets::credentials::CredentialReader;
use futures_util::FutureExt;
use health_report::HealthReport;
use model::machine::{
    InstanceState, MachineState, ManagedHostState, ManagedHostStateSnapshot, ValidationState,
    get_display_ids,
};
use model::power_manager::PowerHandlingOutcome;
use model::resource_pool::common::CommonPools;
use sqlx::PgConnection;
use tokio::sync::Semaphore;
use tracing::instrument;

use crate::cfg::file::{BomValidationConfig, FirmwareConfig, MachineValidationConfig, TimePeriod};
use crate::dpf::DpfOperations;
use crate::firmware_downloader::FirmwareDownloader;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::write_ops::MachineWriteOp;
use crate::state_controller::state_handler::{
    StateHandler, StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

// --- Submodules ---

pub mod common;
mod dpf;
mod dpu_machine_state;
mod helpers;
mod host_machine_state;
mod host_reprovision_state;
mod instance_state;
mod machine_validation;
mod managed_host_state;
mod power;
mod sku;

// --- Re-exports (public API) ---

use common::*;
#[allow(unused_imports)]
pub use common::{
    HostHandlerParams, MAX_FIRMWARE_UPGRADE_RETRIES, PowerOptionConfig, ReachabilityParams,
    RebootStatus, check_restart_in_logs, find_explored_refreshed_endpoint,
    handler_host_power_control, host_power_state, identify_dpu, machine_validation_completed,
    rebooted, trigger_reboot_if_needed, try_wait_for_dpu_discovery,
};
pub use dpu_machine_state::DpuMachineStateHandler;
pub use host_machine_state::HostMachineStateHandler;
use host_reprovision_state::HostUpgradeState;
pub use instance_state::{InstanceStateHandler, release_vpc_dpu_loopback};

// --- MachineStateHandler: the dispatch hub ---

pub struct MachineStateHandlerBuilder {
    pub(crate) dpu_up_threshold: chrono::Duration,
    pub(crate) dpu_nic_firmware_initial_update_enabled: bool,
    pub(crate) dpu_nic_firmware_reprovision_update_enabled: bool,
    pub(crate) hardware_models: Option<FirmwareConfig>,
    pub(crate) no_firmware_update_reset_retries: bool,
    pub(crate) reachability_params: ReachabilityParams,
    pub(crate) firmware_downloader: Option<FirmwareDownloader>,
    pub(crate) attestation_enabled: bool,
    pub(crate) upload_limiter: Option<Arc<Semaphore>>,
    pub(crate) machine_validation_config: MachineValidationConfig,
    pub(crate) common_pools: Option<Arc<CommonPools>>,
    pub(crate) bom_validation: BomValidationConfig,
    pub(crate) instance_autoreboot_period: Option<TimePeriod>,
    pub(crate) credential_reader: Option<Arc<dyn CredentialReader>>,
    pub(crate) power_options_config: PowerOptionConfig,
    pub(crate) enable_secure_boot: bool,
    pub(crate) hgx_bmc_gpu_reboot_delay: chrono::Duration,
    pub(crate) dpf_sdk: Option<Arc<dyn DpfOperations>>,
}

impl MachineStateHandlerBuilder {
    pub fn builder() -> Self {
        Self {
            dpu_up_threshold: chrono::Duration::minutes(5),
            dpu_nic_firmware_initial_update_enabled: true,
            dpu_nic_firmware_reprovision_update_enabled: true,
            hardware_models: None,
            reachability_params: ReachabilityParams {
                dpu_wait_time: chrono::Duration::zero(),
                power_down_wait: chrono::Duration::zero(),
                failure_retry_time: chrono::Duration::zero(),
                scout_reporting_timeout: chrono::Duration::zero(),
                uefi_boot_wait: chrono::Duration::zero(),
            },
            firmware_downloader: None,
            no_firmware_update_reset_retries: false,
            attestation_enabled: false,
            upload_limiter: None,
            machine_validation_config: MachineValidationConfig {
                enabled: true,
                ..MachineValidationConfig::default()
            },
            common_pools: None,
            bom_validation: BomValidationConfig::default(),
            instance_autoreboot_period: None,
            credential_reader: None,
            power_options_config: PowerOptionConfig {
                enabled: true,
                next_try_duration_on_success: chrono::Duration::minutes(0),
                next_try_duration_on_failure: chrono::Duration::minutes(0),
                wait_duration_until_host_reboot: chrono::Duration::minutes(0),
            },
            enable_secure_boot: false,
            hgx_bmc_gpu_reboot_delay: chrono::Duration::seconds(30),
            dpf_sdk: None,
        }
    }

    pub fn dpf_sdk(mut self, dpf_sdk: Option<Arc<dyn DpfOperations>>) -> Self {
        self.dpf_sdk = dpf_sdk;
        self
    }

    pub fn credential_reader(mut self, credential_reader: Arc<dyn CredentialReader>) -> Self {
        self.credential_reader = Some(credential_reader);
        self
    }
    pub fn dpu_up_threshold(mut self, dpu_up_threshold: chrono::Duration) -> Self {
        self.dpu_up_threshold = dpu_up_threshold;
        self
    }

    #[cfg(test)]
    pub fn dpu_nic_firmware_initial_update_enabled(
        mut self,
        dpu_nic_firmware_initial_update_enabled: bool,
    ) -> Self {
        self.dpu_nic_firmware_initial_update_enabled = dpu_nic_firmware_initial_update_enabled;
        self
    }

    pub fn dpu_nic_firmware_reprovision_update_enabled(
        mut self,
        dpu_nic_firmware_reprovision_update_enabled: bool,
    ) -> Self {
        self.dpu_nic_firmware_reprovision_update_enabled =
            dpu_nic_firmware_reprovision_update_enabled;
        self
    }

    #[cfg(test)]
    pub fn reachability_params(mut self, reachability_params: ReachabilityParams) -> Self {
        self.reachability_params = reachability_params;
        self
    }

    pub fn dpu_wait_time(mut self, dpu_wait_time: chrono::Duration) -> Self {
        self.reachability_params.dpu_wait_time = dpu_wait_time;
        self
    }

    pub fn dpu_enable_secure_boot(mut self, dpu_enable_secure_boot: bool) -> Self {
        self.enable_secure_boot = dpu_enable_secure_boot;
        self
    }

    pub fn power_down_wait(mut self, power_down_wait: chrono::Duration) -> Self {
        self.reachability_params.power_down_wait = power_down_wait;
        self
    }

    pub fn failure_retry_time(mut self, failure_retry_time: chrono::Duration) -> Self {
        self.reachability_params.failure_retry_time = failure_retry_time;
        self
    }

    pub fn scout_reporting_timeout(mut self, scout_reporting_timeout: chrono::Duration) -> Self {
        self.reachability_params.scout_reporting_timeout = scout_reporting_timeout;
        self
    }

    pub fn uefi_boot_wait(mut self, uefi_boot_wait: chrono::Duration) -> Self {
        self.reachability_params.uefi_boot_wait = uefi_boot_wait;
        self
    }

    pub fn hardware_models(mut self, hardware_models: FirmwareConfig) -> Self {
        self.hardware_models = Some(hardware_models);
        self
    }

    pub fn firmware_downloader(mut self, firmware_downloader: &FirmwareDownloader) -> Self {
        self.firmware_downloader = Some(firmware_downloader.clone());
        self
    }

    pub fn attestation_enabled(mut self, attestation_enabled: bool) -> Self {
        self.attestation_enabled = attestation_enabled;
        self
    }

    pub fn upload_limiter(mut self, upload_limiter: Arc<Semaphore>) -> Self {
        self.upload_limiter = Some(upload_limiter);
        self
    }

    pub fn machine_validation_config(
        mut self,
        machine_validation_config: MachineValidationConfig,
    ) -> Self {
        self.machine_validation_config = machine_validation_config;
        self
    }

    pub fn common_pools(mut self, common_pools: Arc<CommonPools>) -> Self {
        self.common_pools = Some(common_pools);
        self
    }

    pub fn bom_validation(mut self, bom_validation: BomValidationConfig) -> Self {
        self.bom_validation = bom_validation;
        self
    }

    pub fn no_firmware_update_reset_retries(
        mut self,
        no_firmware_update_reset_retries: bool,
    ) -> Self {
        self.no_firmware_update_reset_retries = no_firmware_update_reset_retries;
        self
    }

    pub fn instance_autoreboot_period(mut self, period: Option<TimePeriod>) -> Self {
        self.instance_autoreboot_period = period;
        self
    }

    pub fn power_options_config(mut self, config: PowerOptionConfig) -> Self {
        self.power_options_config = config;
        self
    }

    pub fn build(self) -> MachineStateHandler {
        MachineStateHandler::new(self)
    }
}

/// The actual Machine State handler
#[derive(Debug, Clone)]
pub struct MachineStateHandler {
    host_handler: HostMachineStateHandler,
    pub dpu_handler: DpuMachineStateHandler,
    instance_handler: InstanceStateHandler,
    dpu_up_threshold: Duration,
    reachability_params: ReachabilityParams,
    host_upgrade: Arc<HostUpgradeState>,
    power_options_config: PowerOptionConfig,
    enable_secure_boot: bool,
}

impl MachineStateHandler {
    fn new(builder: MachineStateHandlerBuilder) -> Self {
        let host_upgrade = Arc::new(HostUpgradeState {
            parsed_hosts: Arc::new(builder.hardware_models.clone().unwrap_or_default()),
            downloader: builder.firmware_downloader.unwrap_or_default(),
            upload_limiter: builder
                .upload_limiter
                .unwrap_or(Arc::new(tokio::sync::Semaphore::new(5))),
            no_firmware_update_reset_retries: builder.no_firmware_update_reset_retries,
            instance_autoreboot_period: builder.instance_autoreboot_period,
            upgrade_script_state: Default::default(),
            credential_reader: builder.credential_reader,
            async_firmware_uploader: Arc::new(Default::default()),
            hgx_bmc_gpu_reboot_delay: builder
                .hgx_bmc_gpu_reboot_delay
                .to_std()
                .unwrap_or(tokio::time::Duration::from_secs(30)),
        });
        MachineStateHandler {
            dpu_up_threshold: builder.dpu_up_threshold,
            host_handler: HostMachineStateHandler::new(HostHandlerParams {
                attestation_enabled: builder.attestation_enabled,
                reachability_params: builder.reachability_params,
                machine_validation_config: builder.machine_validation_config,
                bom_validation: builder.bom_validation,
            }),
            dpu_handler: DpuMachineStateHandler::new(
                builder.dpu_nic_firmware_initial_update_enabled,
                builder.hardware_models.clone().unwrap_or_default(),
                builder.reachability_params,
                builder.enable_secure_boot,
                builder.dpf_sdk.clone(),
            ),
            instance_handler: InstanceStateHandler::new(
                builder.attestation_enabled,
                builder.reachability_params,
                builder.common_pools,
                host_upgrade.clone(),
                builder.hardware_models.clone().unwrap_or_default(),
                builder.enable_secure_boot,
                builder.dpf_sdk.clone(),
            ),
            reachability_params: builder.reachability_params,
            host_upgrade,
            power_options_config: builder.power_options_config,
            enable_secure_boot: builder.enable_secure_boot,
        }
    }

    fn record_metrics(
        &self,
        state: &mut ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    ) {
        for dpu_snapshot in state.dpu_snapshots.iter() {
            let fw_version = dpu_snapshot
                .hardware_info
                .as_ref()
                .and_then(|hi| hi.dpu_info.as_ref().map(|di| di.firmware_version.clone()));
            if let Some(fw_version) = fw_version {
                *ctx.metrics
                    .dpu_firmware_versions
                    .entry(fw_version)
                    .or_default() += 1;
            }

            for mut component in dpu_snapshot
                .inventory
                .as_ref()
                .map(|i| i.components.clone())
                .unwrap_or_default()
            {
                component.url = String::new();
                *ctx.metrics
                    .machine_inventory_component_versions
                    .entry(component)
                    .or_default() += 1;
            }

            ctx.metrics.dpus_healthy += if dpu_snapshot
                .dpu_agent_health_report
                .as_ref()
                .map(|health| health.alerts.is_empty())
                .unwrap_or(false)
            {
                1
            } else {
                0
            };
            if let Some(report) = dpu_snapshot.dpu_agent_health_report.as_ref() {
                for alert in report.alerts.iter() {
                    *ctx.metrics
                        .dpu_health_probe_alerts
                        .entry((alert.id.clone(), alert.target.clone()))
                        .or_default() += 1;
                }
            }
            if let Some(observation) = dpu_snapshot.network_status_observation.as_ref() {
                if let Some(agent_version) = observation.agent_version.as_ref() {
                    *ctx.metrics
                        .agent_versions
                        .entry(agent_version.clone())
                        .or_default() += 1;
                }
                if Utc::now().signed_duration_since(observation.observed_at)
                    <= self.dpu_up_threshold
                {
                    ctx.metrics.dpus_up += 1;
                }

                *ctx.metrics
                    .client_certificate_expiry
                    .entry(observation.machine_id.to_string())
                    .or_default() = observation.client_certificate_expiry;
            }
        }

        ctx.metrics.machine_id = state.host_snapshot.id.to_string();
        ctx.metrics.is_usable_as_instance = state.is_usable_as_instance(false).is_ok();
        ctx.metrics.num_gpus = state
            .host_snapshot
            .hardware_info
            .as_ref()
            .map(|info| info.gpus.len())
            .unwrap_or_default();
        ctx.metrics.in_use_by_tenant = state
            .instance
            .as_ref()
            .map(|instance| instance.config.tenant.tenant_organization_id.clone());
        ctx.metrics.is_host_bios_password_set =
            state.host_snapshot.bios_password_set_time.is_some();
        ctx.metrics.sku = state.host_snapshot.hw_sku.clone();
        ctx.metrics.sku_device_type = state.host_snapshot.hw_sku_device_type.clone();

        let suppress_alerts =
            health_report::HealthAlertClassification::suppress_external_alerting();
        for alert in state.aggregate_health.alerts.iter() {
            ctx.metrics
                .health_probe_alerts
                .insert((alert.id.clone(), alert.target.clone()));
            for c in alert.classifications.iter() {
                ctx.metrics.health_alert_classifications.insert(c.clone());
                if *c == suppress_alerts {
                    ctx.metrics.alerts_suppressed = true;
                }
            }
        }

        ctx.metrics.num_merge_overrides = state.host_snapshot.health_report_overrides.merges.len();
        ctx.metrics.replace_override_enabled = state
            .host_snapshot
            .health_report_overrides
            .replace
            .is_some();
    }

    fn record_health_history(
        &self,
        mh_snapshot: &mut ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    ) {
        ctx.pending_db_writes
            .push(MachineWriteOp::PersistMachineHealthHistory {
                machine_id: mh_snapshot.host_snapshot.id,
                health_report: mh_snapshot.aggregate_health.clone(),
            })
    }

    async fn clear_dpu_reprovision(
        mh_snaphost: &ManagedHostStateSnapshot,
        txn: &mut PgConnection,
    ) -> Result<(), StateHandlerError> {
        db::machine::remove_health_report_override(
            txn,
            &mh_snaphost.host_snapshot.id,
            health_report::OverrideMode::Merge,
            model::machine_update_module::HOST_UPDATE_HEALTH_REPORT_SOURCE,
        )
        .await?;

        for dpu_snapshot in &mh_snaphost.dpu_snapshots {
            db::machine::clear_dpu_reprovisioning_request(txn, &dpu_snapshot.id, false).await?;
        }

        Ok(())
    }

    async fn clear_scout_timeout_alert(
        txn: &mut PgConnection,
        host_machine_id: &MachineId,
    ) -> Result<(), StateHandlerError> {
        db::machine::remove_health_report_override(
            txn,
            host_machine_id,
            health_report::OverrideMode::Merge,
            "scout",
        )
        .await?;
        Ok(())
    }

    async fn clear_host_reprovision(
        mh_snaphost: &ManagedHostStateSnapshot,
        txn: &mut PgConnection,
    ) -> Result<(), StateHandlerError> {
        db::host_machine_update::clear_host_reprovisioning_request(
            txn,
            &mh_snaphost.host_snapshot.id,
        )
        .await?;
        Ok(())
    }

    async fn clear_host_update_alert_and_reprov(
        mh_snaphost: &ManagedHostStateSnapshot,
        txn: &mut PgConnection,
    ) -> Result<(), StateHandlerError> {
        Self::clear_dpu_reprovision(mh_snaphost, txn).await?;
        Self::clear_host_reprovision(mh_snaphost, txn).await
    }

    /// The core state machine dispatch.
    ///
    /// This function is the heart of the machine state controller. It matches
    /// on the current `ManagedHostState` and delegates to the appropriate
    /// submodule for handling.
    async fn attempt_state_transition(
        &self,
        host_machine_id: &MachineId,
        mh_snapshot: &mut ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let mh_state = mh_snapshot.managed_state.clone();

        // If it's been more than 5 minutes since DPU reported status, consider it unhealthy
        for dpu_snapshot in &mh_snapshot.dpu_snapshots {
            if let Some(dpu_health) = dpu_snapshot.dpu_agent_health_report.as_ref() {
                if !dpu_health.alerts.is_empty() {
                    continue;
                }
                if let Some(observation) = &dpu_snapshot.network_status_observation {
                    let observed_at = observation.observed_at;
                    let since_last_seen = Utc::now().signed_duration_since(observed_at);
                    if since_last_seen > self.dpu_up_threshold {
                        let message = format!("Last seen over {} ago", self.dpu_up_threshold);
                        let dpu_machine_id = &dpu_snapshot.id;
                        let health_report = HealthReport::heartbeat_timeout(
                            "forge-dpu-agent".to_string(),
                            "forge-dpu-agent".to_string(),
                            message,
                            true,
                            false,
                        );

                        let mut txn = ctx.services.db_pool.begin().await?;
                        db::machine::update_dpu_agent_health_report(
                            &mut txn,
                            dpu_machine_id,
                            &health_report,
                        )
                        .await?;

                        tracing::warn!(
                        host_machine_id = %host_machine_id,
                        dpu_machine_id = %dpu_machine_id,
                        last_seen = %observed_at,
                        "DPU is not sending network status observations, marking unhealthy");
                        return Ok(StateHandlerOutcome::do_nothing().with_txn(txn));
                    }
                }
            }
        }

        if let Some(outcome) = handle_restart_verification(mh_snapshot, ctx).await? {
            return Ok(outcome);
        }

        if dpu_reprovisioning_needed(&mh_snapshot.dpu_snapshots) {
            let restart_reprov = can_restart_reprovision(
                &mh_snapshot.dpu_snapshots,
                mh_snapshot.host_snapshot.state.version,
            );
            if restart_reprov
                && let Some(next_state) =
                    managed_host_state::dpu_reprovision::start_dpu_reprovision(
                        &mh_state,
                        mh_snapshot,
                        ctx,
                        host_machine_id,
                        self.enable_secure_boot,
                    )
                    .await?
            {
                return Ok(StateHandlerOutcome::transition(next_state));
            }
        }

        // Don't update failed state failure cause everytime. Record first failure cause only.
        if !matches!(mh_state, ManagedHostState::Failed { .. })
            && let Some((machine_id, details)) = get_failed_state(mh_snapshot)
        {
            tracing::error!(
                %machine_id,
                "ManagedHost {}/{} (failed machine: {}) is moved to Failed state with cause: {:?}",
                mh_snapshot.host_snapshot.id,
                get_display_ids(&mh_snapshot.dpu_snapshots),
                machine_id,
                details
            );
            let next_state = match mh_state {
                ManagedHostState::Assigned { .. } => ManagedHostState::Assigned {
                    instance_state: InstanceState::Failed {
                        details,
                        machine_id,
                    },
                },
                _ => ManagedHostState::Failed {
                    details,
                    machine_id,
                    retry_count: 0,
                },
            };
            return Ok(StateHandlerOutcome::transition(next_state));
        }

        // ...and now dispatch out to the target state handler.
        match &mh_state {
            ManagedHostState::VerifyRmsMembership => {
                managed_host_state::verify_rms_membership::handle(
                    host_machine_id,
                    mh_snapshot,
                    ctx,
                    self.enable_secure_boot,
                )
                .await
            }

            ManagedHostState::RegisterRmsMembership => {
                managed_host_state::register_rms_membership::handle(
                    host_machine_id,
                    mh_snapshot,
                    ctx,
                    self.enable_secure_boot,
                )
                .await
            }

            ManagedHostState::DpuDiscoveringState { .. } => {
                if mh_snapshot
                    .host_snapshot
                    .associated_dpu_machine_ids()
                    .is_empty()
                {
                    tracing::info!(
                        machine_id = %host_machine_id,
                        "Skipping to HostInit because machine has no DPUs"
                    );
                    Ok(StateHandlerOutcome::transition(
                        ManagedHostState::HostInit {
                            machine_state: MachineState::WaitingForPlatformConfiguration,
                        },
                    ))
                } else {
                    let mut state_handler_outcome = StateHandlerOutcome::do_nothing();
                    if ctx.services.site_config.force_dpu_nic_mode {
                        return Ok(StateHandlerOutcome::transition(
                            ManagedHostState::HostInit {
                                machine_state: MachineState::WaitingForPlatformConfiguration,
                            },
                        ));
                    }
                    for dpu_snapshot in &mh_snapshot.dpu_snapshots {
                        state_handler_outcome = self
                            .dpu_handler
                            .handle_dpu_discovering_state(mh_snapshot, dpu_snapshot, ctx)
                            .await?;

                        if let outcome @ StateHandlerOutcome::Transition { .. } =
                            state_handler_outcome
                        {
                            return Ok(outcome);
                        }
                    }

                    Ok(state_handler_outcome)
                }
            }

            ManagedHostState::DPUInit { .. } => {
                self.dpu_handler
                    .handle_object_state(host_machine_id, mh_snapshot, &mh_state, ctx)
                    .await
            }

            ManagedHostState::HostInit { .. } => {
                self.host_handler
                    .handle_object_state(host_machine_id, mh_snapshot, &mh_state, ctx)
                    .await
            }

            ManagedHostState::Ready => {
                managed_host_state::ready::handle(
                    host_machine_id,
                    mh_snapshot,
                    ctx,
                    &self.host_handler.host_handler_params,
                    &self.host_upgrade,
                    &self.reachability_params,
                    self.enable_secure_boot,
                    &self.dpu_handler.hardware_models,
                )
                .await
            }

            ManagedHostState::Assigned { instance_state: _ } => {
                self.instance_handler
                    .handle_object_state(host_machine_id, mh_snapshot, &mh_state, ctx)
                    .await
            }

            ManagedHostState::WaitingForCleanup { cleanup_state } => {
                managed_host_state::waiting_for_cleanup::handle(
                    host_machine_id,
                    mh_snapshot,
                    cleanup_state,
                    ctx,
                    &self.reachability_params,
                    &self.host_handler.host_handler_params,
                )
                .await
            }

            ManagedHostState::Created => {
                tracing::error!("Machine just created. We should not be here.");
                Err(StateHandlerError::InvalidHostState(
                    *host_machine_id,
                    Box::new(mh_state.clone()),
                ))
            }

            ManagedHostState::ForceDeletion => {
                tracing::info!(
                    machine_id = %host_machine_id,
                    "Machine is marked for forced deletion. Ignoring.",
                );
                Ok(StateHandlerOutcome::deleted())
            }

            ManagedHostState::Failed {
                details,
                machine_id,
                retry_count,
            } => {
                managed_host_state::failed::handle(
                    host_machine_id,
                    mh_snapshot,
                    details,
                    machine_id,
                    *retry_count,
                    ctx,
                    self.host_handler.host_handler_params.attestation_enabled,
                    &self.reachability_params,
                )
                .await
            }

            ManagedHostState::DPUReprovision { .. } => {
                managed_host_state::dpu_reprovision::handle(
                    mh_snapshot,
                    &self.reachability_params,
                    &self.dpu_handler.hardware_models,
                    self.dpu_handler.dpf_sdk.as_deref(),
                    ctx,
                )
                .await
            }

            ManagedHostState::HostReprovision { .. } => {
                managed_host_state::host_reprovision::handle(
                    &self.host_upgrade,
                    mh_snapshot,
                    ctx,
                    host_machine_id,
                )
                .await
            }

            ManagedHostState::Measuring { measuring_state } => {
                managed_host_state::measuring::handle(
                    measuring_state,
                    host_machine_id,
                    ctx,
                    self.host_handler.host_handler_params.attestation_enabled,
                )
                .await
            }

            ManagedHostState::PostAssignedMeasuring { measuring_state } => {
                managed_host_state::post_assigned_measuring::handle(
                    measuring_state,
                    host_machine_id,
                    ctx,
                    self.host_handler.host_handler_params.attestation_enabled,
                )
                .await
            }

            ManagedHostState::BomValidating {
                bom_validating_state,
            } => {
                sku::handle_bom_validation_state(
                    ctx,
                    &self.host_handler.host_handler_params,
                    mh_snapshot,
                    bom_validating_state,
                )
                .await
            }

            ManagedHostState::Validation { validation_state } => match validation_state {
                ValidationState::MachineValidation { machine_validation } => {
                    machine_validation::handle_machine_validation_state(
                        ctx,
                        machine_validation,
                        &self.host_handler.host_handler_params,
                        mh_snapshot,
                    )
                    .await
                }
            },
        }
    }
}

#[async_trait::async_trait]
impl StateHandler for MachineStateHandler {
    type State = ManagedHostStateSnapshot;
    type ControllerState = ManagedHostState;
    type ObjectId = MachineId;
    type ContextObjects = MachineStateHandlerContextObjects;

    #[instrument(skip_all, fields(object_id=%host_machine_id, state=%_mh_state))]
    async fn handle_object_state(
        &self,
        host_machine_id: &MachineId,
        mh_snapshot: &mut ManagedHostStateSnapshot,
        _mh_state: &Self::ControllerState,
        ctx: &mut StateHandlerContext<Self::ContextObjects>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        if !mh_snapshot
            .host_snapshot
            .associated_dpu_machine_ids()
            .is_empty()
            && mh_snapshot.dpu_snapshots.is_empty()
        {
            return Err(StateHandlerError::GenericError(eyre!(
                "No DPU snapshot found."
            )));
        }

        self.record_metrics(mh_snapshot, ctx);
        self.record_health_history(mh_snapshot, ctx);

        let PowerHandlingOutcome {
            power_options,
            continue_state_machine,
            msg,
        } = match mh_snapshot.host_snapshot.state.value {
            ManagedHostState::Assigned {
                instance_state: InstanceState::Ready,
            } => PowerHandlingOutcome::new(None, true, None),
            _ => {
                if self.power_options_config.enabled {
                    power::handle_power(mh_snapshot, ctx, &self.power_options_config).await?
                } else {
                    PowerHandlingOutcome::new(None, true, None)
                }
            }
        };

        let power_options_pool = ctx.services.db_pool.clone();

        let was_ready = matches!(mh_snapshot.managed_state, ManagedHostState::Ready);

        let mut result = if continue_state_machine {
            self.attempt_state_transition(host_machine_id, mh_snapshot, ctx)
                .await
        } else {
            Ok(StateHandlerOutcome::wait(format!(
                "State machine can't proceed due to power manager. {}",
                msg.unwrap_or_default()
            )))
        };

        if was_ready && let Ok(outcome) = result {
            if matches!(&outcome, StateHandlerOutcome::Transition { .. }) {
                let host_machine_id = *host_machine_id;
                result = Ok(outcome
                    .in_transaction(&ctx.services.db_pool, move |txn| {
                        async move {
                            Self::clear_scout_timeout_alert(txn, &host_machine_id).await?;
                            Ok::<(), StateHandlerError>(())
                        }
                        .boxed()
                    })
                    .await??);
            } else {
                result = Ok(outcome);
            }
        }

        if let Some(power_options) = power_options {
            let mut txn = power_options_pool.begin().await?;
            db::power_options::persist(&power_options, &mut txn).await?;
            txn.commit().await?;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::{Duration, Utc};

    use super::common::get_reboot_cycle;

    #[test]
    fn test_cycle_1() {
        let state_change_time =
            chrono::DateTime::<Utc>::from_str("2024-01-30T11:26:18.261228950+00:00").unwrap();

        let expected_time = state_change_time + Duration::minutes(30);
        let wait_period = Duration::minutes(30);

        let cycle = get_reboot_cycle(expected_time, state_change_time, wait_period).unwrap();
        assert_eq!(cycle, 1);
    }

    #[test]
    fn test_cycle_2() {
        let state_change_time =
            chrono::DateTime::<Utc>::from_str("2024-01-30T11:26:18.261228950+00:00").unwrap();

        let expected_time = state_change_time + Duration::minutes(70);
        let wait_period = Duration::minutes(30);

        let cycle = get_reboot_cycle(expected_time, state_change_time, wait_period).unwrap();
        assert_eq!(cycle, 2);
    }

    #[test]
    fn test_cycle_3() {
        let state_change_time =
            chrono::DateTime::<Utc>::from_str("2024-01-30T11:26:18.261228950+00:00").unwrap();

        let expected_time = state_change_time + Duration::minutes(121);
        let wait_period = Duration::minutes(30);

        let cycle = get_reboot_cycle(expected_time, state_change_time, wait_period).unwrap();
        assert_eq!(cycle, 4);
    }

    #[test]
    fn test_cycle_4() {
        let state_change_time =
            chrono::DateTime::<Utc>::from_str("2024-01-30T11:26:18.261228950+00:00").unwrap();

        let expected_time = state_change_time + Duration::minutes(30);
        let wait_period = Duration::minutes(0);

        let cycle = get_reboot_cycle(expected_time, state_change_time, wait_period).unwrap();
        assert_eq!(cycle, 30);
    }
}
