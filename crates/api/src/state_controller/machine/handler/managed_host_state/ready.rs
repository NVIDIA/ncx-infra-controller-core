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

use std::sync::Arc;

use carbide_uuid::machine::MachineId;
use chrono::Utc;
use futures_util::FutureExt;
use health_report::{HealthReport, OverrideMode};
use itertools::Itertools;
use model::machine::{
    InstanceState, MachineState, ManagedHostState, ManagedHostStateSnapshot, MeasuringState,
    NextStateBFBSupport, ReprovisionState, UefiSetupInfo, UefiSetupState,
};

use super::super::helpers::ReprovisionStateHelper;
use super::super::host_reprovision_state::HostFirmwareScenario;
use super::super::machine_validation::handle_machine_validation_requested;
use super::super::{
    HostHandlerParams, HostUpgradeState, MachineStateHandler, ReachabilityParams,
    dpu_reprovisioning_needed, handler_restart_dpu, host_reprovisioning_requested,
    set_managed_host_topology_update_needed,
};
use super::failed::check_if_should_redo_measurements;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::handler::sku::handle_bom_validation_requested;
use crate::state_controller::machine::write_ops::MachineWriteOp;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle(
    host_machine_id: &MachineId,
    mh_snapshot: &mut ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    host_handler_params: &HostHandlerParams,
    host_upgrade: &Arc<HostUpgradeState>,
    reachability_params: &ReachabilityParams,
    enable_secure_boot: bool,
    _hardware_models: &crate::cfg::file::FirmwareConfig,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    if let Some(outcome) =
        handle_scout_heartbeat_timeout(mh_snapshot, ctx, reachability_params).await?
    {
        return Ok(outcome);
    }

    // Check if instance to be created.
    if mh_snapshot.instance.is_some() {
        // Instance is requested by user. Let's configure it.
        let mut txn = ctx.services.db_pool.begin().await?;

        // Clear if any reprovision (dpu or host) is set due to race scenario.
        MachineStateHandler::clear_host_update_alert_and_reprov(mh_snapshot, &mut txn).await?;

        let mut next_state = ManagedHostState::Assigned {
            instance_state: InstanceState::DpaProvisioning,
        };

        if !ctx.services.site_config.is_dpa_enabled() {
            // If DPA is not enabled, we don't need to do any DPA provisioning.
            // So go directly to WaitingForDpaToBeReady state, where we will change
            // the network status of our DPUs.
            next_state = ManagedHostState::Assigned {
                instance_state: InstanceState::WaitingForDpaToBeReady,
            };
        }
        return Ok(StateHandlerOutcome::transition(next_state).with_txn(txn));
    }

    if let Some(outcome) =
        handle_bom_validation_requested(host_handler_params, mh_snapshot, ctx.services).await?
    {
        return Ok(outcome);
    }

    if host_reprovisioning_requested(mh_snapshot) {
        let outcome = host_upgrade
            .handle_host_reprovision(
                mh_snapshot,
                ctx,
                host_machine_id,
                HostFirmwareScenario::Ready,
            )
            .await?;
        if matches!(outcome, StateHandlerOutcome::Transition { .. }) {
            let health_report =
            crate::machine_update_manager::machine_update_module::create_host_update_health_report_hostfw();
            let host_machine_id = *host_machine_id;

            // The health report alert gets generated here, the machine update manager
            // retains responsibilty for clearing it when we're done.
            return Ok(outcome
                .in_transaction(&ctx.services.db_pool, move |txn| {
                    async move {
                        db::machine::insert_health_report_override(
                            txn,
                            &host_machine_id,
                            health_report::OverrideMode::Merge,
                            &health_report,
                            false,
                        )
                        .await
                    }
                    .boxed()
                })
                .await??);
        } else {
            return Ok(outcome);
        }
    }
    if let Some(outcome) =
        handle_machine_validation_requested(ctx.services, mh_snapshot, false).await?
    {
        return Ok(outcome);
    }

    // Check if DPU reprovisioning is requested
    if dpu_reprovisioning_needed(&mh_snapshot.dpu_snapshots) {
        let mut dpus_for_reprov = vec![];
        for dpu_snapshot in &mh_snapshot.dpu_snapshots {
            if dpu_snapshot.reprovision_requested.is_some() {
                handler_restart_dpu(
                    dpu_snapshot,
                    ctx,
                    mh_snapshot.host_snapshot.dpf.used_for_ingestion,
                )
                .await?;
                ctx.pending_db_writes
                    .push(MachineWriteOp::UpdateDpuReprovisionStartTime {
                        machine_id: dpu_snapshot.id,
                        time: Utc::now(),
                    });
                dpus_for_reprov.push(dpu_snapshot);
            }
        }

        set_managed_host_topology_update_needed(
            ctx.pending_db_writes,
            &mh_snapshot.host_snapshot,
            &dpus_for_reprov,
        );

        let reprov_state = ReprovisionState::next_substate_based_on_bfb_support(
            enable_secure_boot,
            mh_snapshot,
            ctx.services.site_config.dpf.enabled,
        );

        let next_state = reprov_state.next_state_with_all_dpus_updated(
            &mh_snapshot.managed_state,
            &mh_snapshot.dpu_snapshots,
            dpus_for_reprov.iter().map(|x| &x.id).collect_vec(),
        )?;

        let health_override = crate::machine_update_manager::machine_update_module::create_host_update_health_report_dpufw();

        // Mark the Host as in update.
        let mut txn = ctx.services.db_pool.begin().await?;
        db::machine::insert_health_report_override(
            &mut txn,
            host_machine_id,
            health_report::OverrideMode::Merge,
            &health_override,
            false,
        )
        .await?;
        return Ok(StateHandlerOutcome::transition(next_state).with_txn(txn));
    }

    // Check to see if measurement machine (i.e. attestation) state has changed
    // if so, just place it into the measuring state and let it be handled inside
    // the measurement state
    if host_handler_params.attestation_enabled
        && check_if_should_redo_measurements(
            &mh_snapshot.host_snapshot.id,
            &mut ctx.services.db_reader,
        )
        .await?
    {
        return Ok(StateHandlerOutcome::transition(
            ManagedHostState::Measuring {
                measuring_state: MeasuringState::WaitingForMeasurements, // let's just start from the beginning
            },
        ));
    }

    // This feature has only been tested thoroughly on Dells and Lenovos
    if (mh_snapshot.host_snapshot.bmc_vendor().is_dell()
        || mh_snapshot.host_snapshot.bmc_vendor().is_lenovo())
        && mh_snapshot.host_snapshot.bios_password_set_time.is_none()
    {
        tracing::info!(
            "transitioning legacy {} host {} to UefiSetupState::UnlockHost while it is in ManagedHostState::Ready so that the BIOS password can be configured",
            mh_snapshot.host_snapshot.bmc_vendor(),
            mh_snapshot.host_snapshot.id
        );
        return Ok(StateHandlerOutcome::transition(
            ManagedHostState::HostInit {
                machine_state: MachineState::UefiSetup {
                    uefi_setup_info: UefiSetupInfo {
                        uefi_password_jid: None,
                        uefi_setup_state: UefiSetupState::UnlockHost,
                    },
                },
            },
        ));
    }

    Ok(StateHandlerOutcome::do_nothing())
}

async fn handle_scout_heartbeat_timeout(
    mh_snapshot: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    reachability_params: &ReachabilityParams,
) -> Result<Option<StateHandlerOutcome<ManagedHostState>>, StateHandlerError> {
    let host_machine_id = &mh_snapshot.host_snapshot.id;
    let Some(last_scout_contact) = mh_snapshot.host_snapshot.last_scout_contact_time else {
        return Ok(None);
    };

    let since_last_contact = Utc::now().signed_duration_since(last_scout_contact);
    let timeout_threshold = reachability_params.scout_reporting_timeout;
    let scout_timeout_alert_exists = mh_snapshot
        .host_snapshot
        .health_report_overrides
        .merges
        .contains_key("scout");

    if since_last_contact >= timeout_threshold {
        ctx.metrics.host_with_scout_heartbeat_timeout = Some(host_machine_id.to_string());
    }

    if since_last_contact >= timeout_threshold && !scout_timeout_alert_exists {
        let message = format!("Last scout heartbeat over {timeout_threshold} ago");
        let host_health = &ctx.services.site_config.host_health;
        let health_report = HealthReport::heartbeat_timeout(
            "scout".to_string(),
            "scout".to_string(),
            message,
            host_health.prevent_allocations_on_scout_heartbeat_timeout,
            host_health.suppress_external_alerting_on_scout_heartbeat_timeout,
        );

        let mut txn = ctx.services.db_pool.begin().await?;
        db::machine::insert_health_report_override(
            &mut txn,
            host_machine_id,
            OverrideMode::Merge,
            &health_report,
            false,
        )
        .await?;
        tracing::warn!(
            host_machine_id = %host_machine_id,
            last_scout_contact = %last_scout_contact,
            timeout_threshold = %timeout_threshold,
            "Scout heartbeat timeout detected, adding health alert"
        );
        return Ok(Some(StateHandlerOutcome::do_nothing().with_txn(txn)));
    }

    if since_last_contact < timeout_threshold && scout_timeout_alert_exists {
        let mut txn = ctx.services.db_pool.begin().await?;
        MachineStateHandler::clear_scout_timeout_alert(&mut txn, host_machine_id).await?;
        tracing::info!(
            host_machine_id = %host_machine_id,
            last_scout_contact = %last_scout_contact,
            timeout_threshold = %timeout_threshold,
            "Scout heartbeat recovered, removing health alert"
        );
        return Ok(Some(StateHandlerOutcome::do_nothing().with_txn(txn)));
    }

    Ok(None)
}
