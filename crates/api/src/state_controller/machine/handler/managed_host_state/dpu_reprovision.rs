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

use carbide_uuid::machine::MachineId;
use chrono::Utc;
use eyre::eyre;
use forge_secrets::credentials::{BmcCredentialType, CredentialKey};
use itertools::Itertools;
use libredfish::SystemPowerControl;
use model::machine::{
    InstanceState, Machine, MachineNextStateResolver, ManagedHostState, ManagedHostStateSnapshot,
    NextStateBFBSupport, ReprovisionState,
};

use super::super::helpers::{NextState, ReprovisionStateHelper, all_equal};
use super::super::host_machine_state::managed_host_network_config_version_synced_and_dpu_healthy;
use super::super::{
    ReachabilityParams, check_fw_component_version, handle_bfb_install_state,
    handler_host_power_control, handler_restart_dpu, host_power_state, is_dpu_up,
    set_managed_host_topology_update_needed, trigger_reboot_if_needed, try_wait_for_dpu_discovery,
    wait,
};
use crate::cfg::file::FirmwareConfig;
use crate::dpf::DpfOperations;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::write_ops::MachineWriteOp;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle(
    mh_snapshot: &mut ManagedHostStateSnapshot,
    reachability_params: &ReachabilityParams,
    dpu_handler_hardware_models: &FirmwareConfig,
    dpf_sdk: Option<&dyn DpfOperations>,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    for dpu_snapshot in &mh_snapshot.dpu_snapshots {
        // TODO: Optimization Possible: We can have another outcome something like
        // TransitionNotPossible. This will be valid for the sync states (States where
        // we wait for all DPUs to come in same state). If return value is
        // TransitionNotPossible, means at least one DPU is not in ready to move into
        // next state, thus no point of checking for next DPU. In this case, just break
        // the loop.
        if let outcome @ StateHandlerOutcome::Transition { .. } = handle_dpu_reprovision(
            mh_snapshot,
            reachability_params,
            &MachineNextStateResolver,
            dpu_snapshot,
            ctx,
            dpu_handler_hardware_models,
            dpf_sdk,
        )
        .await?
        {
            return Ok(outcome);
        }
    }
    Ok(StateHandlerOutcome::do_nothing())
}

pub(crate) async fn start_dpu_reprovision(
    managed_state: &ManagedHostState,
    state: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    host_machine_id: &MachineId,
    enable_secure_boot: bool,
) -> Result<Option<ManagedHostState>, StateHandlerError> {
    let next_state: Option<ManagedHostState>;

    let dpus_for_reprov = state
        .dpu_snapshots
        .iter()
        .filter(|x| x.reprovision_requested.is_some())
        .collect_vec();

    match managed_state {
        ManagedHostState::Assigned {
            instance_state: InstanceState::DPUReprovision { .. } | InstanceState::Failed { .. },
        } => {
            // If we are here means already reprovision is going on, as validated by
            // can_restart_reprovision fucntion.
            next_state = handle_restart_dpu_reprovision_assigned_state(
                state,
                ctx,
                host_machine_id,
                &dpus_for_reprov,
                enable_secure_boot,
            )
            .await?;

            for dpu_id in dpus_for_reprov.iter().map(|d| d.id) {
                ctx.pending_db_writes
                    .push(MachineWriteOp::ClearFailureDetails { machine_id: dpu_id });
            }
        }
        ManagedHostState::DPUReprovision { .. } => {
            set_managed_host_topology_update_needed(
                ctx.pending_db_writes,
                &state.host_snapshot,
                &dpus_for_reprov,
            );

            next_state = Some(
                ReprovisionState::next_substate_based_on_bfb_support(
                    enable_secure_boot,
                    state,
                    ctx.services.site_config.dpf.enabled,
                )
                .next_state_with_all_dpus_updated(
                    &state.managed_state,
                    &state.dpu_snapshots,
                    dpus_for_reprov.iter().map(|x| &x.id).collect_vec(),
                )?,
            );
        }
        _ => {
            next_state = None;
        }
    };

    if next_state.is_some() {
        // Restart all DPUs, sit back and relax.
        for dpu in dpus_for_reprov {
            ctx.pending_db_writes
                .push(MachineWriteOp::UpdateDpuReprovisionStartTime {
                    machine_id: dpu.id,
                    time: Utc::now(),
                });
            handler_restart_dpu(dpu, ctx, state.host_snapshot.dpf.used_for_ingestion).await?;
        }
        return Ok(next_state);
    }

    Ok(None)
}

pub(crate) async fn handle_restart_dpu_reprovision_assigned_state(
    state: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    host_machine_id: &MachineId,
    dpus_for_reprov: &[&Machine],
    enable_secure_boot: bool,
) -> Result<Option<ManagedHostState>, StateHandlerError> {
    // User approval must have received, otherwise reprovision has not
    // started.
    if let Err(err) = handler_host_power_control(state, ctx, SystemPowerControl::ForceRestart).await
    {
        tracing::error!(%host_machine_id, "Host reboot failed with error: {err}");
    }
    set_managed_host_topology_update_needed(
        ctx.pending_db_writes,
        &state.host_snapshot,
        dpus_for_reprov,
    );

    let reprov_state = ReprovisionState::next_substate_based_on_bfb_support(
        enable_secure_boot,
        state,
        ctx.services.site_config.dpf.enabled,
    );
    Ok(Some(reprov_state.next_state_with_all_dpus_updated(
        &state.managed_state,
        &state.dpu_snapshots,
        dpus_for_reprov.iter().map(|x| &x.id).collect_vec(),
    )?))
}

pub(crate) async fn handle_dpu_reprovision(
    state: &ManagedHostStateSnapshot,
    reachability_params: &ReachabilityParams,
    next_state_resolver: &impl NextState,
    dpu_snapshot: &Machine,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    hardware_models: &FirmwareConfig,
    dpf_sdk: Option<&dyn DpfOperations>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let dpu_machine_id = &dpu_snapshot.id;
    let reprovision_state = state
        .managed_state
        .as_reprovision_state(dpu_machine_id)
        .ok_or_else(|| StateHandlerError::MissingData {
            object_id: dpu_machine_id.to_string(),
            missing: "dpu_state",
        })?;

    match reprovision_state {
        ReprovisionState::DpfStates { substate } => {
            let dpf = dpf_sdk.ok_or_else(|| {
                StateHandlerError::GenericError(eyre::eyre!(
                    "DPF reprovision state reached but DPF is not configured"
                ))
            })?;
            super::super::dpf::handle_dpf_state(state, dpu_snapshot, substate, ctx, dpf).await
        }
        ReprovisionState::InstallDpuOs { substate } => {
            handle_bfb_install_state(
                state,
                substate.clone(),
                dpu_snapshot,
                ctx,
                next_state_resolver,
            )
            .await
        }
        ReprovisionState::BmcFirmwareUpgrade { .. } => Ok(StateHandlerOutcome::transition(
            next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
        )),
        ReprovisionState::FirmwareUpgrade => {
            // Firmware upgrade is going on. Lets wait for it to over.
            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
            ))
        }
        ReprovisionState::WaitingForNetworkInstall => {
            if let Some(dpu_id) =
                try_wait_for_dpu_discovery(state, reachability_params, ctx, true, dpu_machine_id)
                    .await?
            {
                // Return Wait.
                return Ok(StateHandlerOutcome::wait(format!(
                    "DPU discovery for {dpu_id} is still not completed."
                )));
            }

            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
            ))
        }
        ReprovisionState::PoweringOffHost => {
            let dpus_states_for_reprov = &state
                .dpu_snapshots
                .iter()
                .filter_map(|x| {
                    if x.reprovision_requested.is_some() {
                        state.managed_state.as_reprovision_state(dpu_machine_id)
                    } else {
                        None
                    }
                })
                .collect_vec();

            if !all_equal(dpus_states_for_reprov)? {
                return Ok(StateHandlerOutcome::wait(
                    "Waiting for DPUs to come in PoweringOffHost state.".to_string(),
                ));
            }

            handler_host_power_control(state, ctx, SystemPowerControl::ForceOff).await?;
            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
            ))
        }
        ReprovisionState::PowerDown => {
            let basetime = state
                .host_snapshot
                .last_reboot_requested
                .as_ref()
                .map(|x| x.time)
                .unwrap_or(state.host_snapshot.state.version.timestamp());

            if wait(&basetime, reachability_params.power_down_wait) {
                return Ok(StateHandlerOutcome::do_nothing());
            }

            let redfish_client = ctx
                .services
                .create_redfish_client_from_machine(&state.host_snapshot)
                .await?;
            let power_state = host_power_state(redfish_client.as_ref()).await?;

            // Host is not powered-off yet. Try again.
            if power_state != libredfish::PowerState::Off {
                tracing::error!(
                    "Machine {} is still not power-off state. Turning off for host again.",
                    state.host_snapshot.id
                );
                handler_host_power_control(state, ctx, SystemPowerControl::ForceOff).await?;

                return Ok(StateHandlerOutcome::wait(format!(
                    "Host {} is not still powered off. Trying again.",
                    state.host_snapshot.id
                )));
            }

            // Mark all re-provisioned DPUs for topology update.
            let dpus_snapshots_for_reprov = &state
                .dpu_snapshots
                .iter()
                .filter(|x| x.reprovision_requested.is_some())
                .collect_vec();

            set_managed_host_topology_update_needed(
                ctx.pending_db_writes,
                &state.host_snapshot,
                dpus_snapshots_for_reprov,
            );

            handler_host_power_control(state, ctx, SystemPowerControl::On).await?;
            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
            ))
        }
        ReprovisionState::BufferTime => Ok(StateHandlerOutcome::transition(
            next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
        )),
        ReprovisionState::VerifyFirmareVersions => {
            if let Some(outcome) =
                check_fw_component_version(ctx, dpu_snapshot, hardware_models).await?
            {
                return Ok(outcome);
            }

            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state(
                    &state.managed_state,
                    dpu_machine_id,
                    &state.host_snapshot,
                )?,
            ))
        }
        ReprovisionState::WaitingForNetworkConfig => {
            let dpus_states_for_reprov = &state
                .dpu_snapshots
                .iter()
                .filter_map(|x| {
                    if x.reprovision_requested.is_some() {
                        state.managed_state.as_reprovision_state(dpu_machine_id)
                    } else {
                        None
                    }
                })
                .collect_vec();

            if !all_equal(dpus_states_for_reprov)? {
                return Ok(StateHandlerOutcome::wait(
                    "Waiting for DPUs to come in WaitingForNetworkConfig state.".to_string(),
                ));
            }
            for dsnapshot in &state.dpu_snapshots {
                if !is_dpu_up(state, dsnapshot) {
                    let msg = format!("Waiting for DPU {} to come up", dsnapshot.id);
                    tracing::warn!("{msg}");

                    let mut reboot_status = None;
                    // Reboot only dpu for which handler is called.
                    if dpu_snapshot.id == dsnapshot.id {
                        reboot_status = Some(
                            trigger_reboot_if_needed(
                                dsnapshot,
                                state,
                                None,
                                reachability_params,
                                ctx,
                            )
                            .await?,
                        );
                    }

                    return Ok(StateHandlerOutcome::wait(format!(
                        "{msg};\nreboot_status: {reboot_status:#?}"
                    )));
                }

                if !managed_host_network_config_version_synced_and_dpu_healthy(dsnapshot) {
                    tracing::warn!("Waiting for network to be ready for DPU {}", dsnapshot.id);

                    // we requested a DPU reboot in ReprovisionState::WaitingForNetworkInstall
                    // let the trigger_reboot_if_needed determine if we are stuck here
                    // (based on how long it has been since the last requested reboot)
                    let mut reboot_status = None;
                    // Reboot only dpu for which handler is called.
                    if dpu_snapshot.id == dsnapshot.id {
                        reboot_status = Some(
                            trigger_reboot_if_needed(
                                dsnapshot,
                                state,
                                None,
                                reachability_params,
                                ctx,
                            )
                            .await?,
                        );
                    }
                    // TODO: Make is_network_ready give us more details as a string
                    return Ok(StateHandlerOutcome::wait(format!(
                        "Waiting for DPU {} to sync network config/become healthy;\nreboot status: {reboot_status:#?}",
                        dsnapshot.id
                    )));
                }
            }

            let mut txn = ctx.services.db_pool.begin().await?;

            // Clear reprovisioning state.
            for dpu_snapshot in &state.dpu_snapshots {
                db::machine::clear_dpu_reprovisioning_request(&mut txn, &dpu_snapshot.id, false)
                    .await?;
            }

            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
            )
            .with_txn(txn))
        }
        ReprovisionState::RebootHostBmc => {
            // Work around for FORGE-3864
            // A NIC FW update from 24.39.2048 to 24.41.1000 can cause the Redfish service to become unavailable on Lenovos.
            // Forge initiates a NIC FW update in ReprovisionState::FirmwareUpgrade
            // At this point, all of the host's DPU have finished the NIC FW Update, been power cycled, and the ARM has come up on the DPU.
            if state.host_snapshot.bmc_vendor().is_lenovo() {
                tracing::info!(
                    "Initiating BMC reset of lenovo machine {}",
                    state.host_snapshot.id
                );

                let redfish_client = ctx
                    .services
                    .create_redfish_client_from_machine(&state.host_snapshot)
                    .await?;

                if let Err(redfish_error) = redfish_client.bmc_reset().await {
                    tracing::warn!(
                        "Failed to reboot BMC for {} through redfish, will try ipmitool: {redfish_error}",
                        &state.host_snapshot.id
                    );

                    let bmc_mac_address = state.host_snapshot.bmc_info.mac.ok_or_else(|| {
                        StateHandlerError::MissingData {
                            object_id: state.host_snapshot.id.to_string(),
                            missing: "bmc_mac",
                        }
                    })?;

                    let bmc_ip_address = state
                        .host_snapshot
                        .bmc_info
                        .ip
                        .clone()
                        .ok_or_else(|| StateHandlerError::MissingData {
                            object_id: state.host_snapshot.id.to_string(),
                            missing: "bmc_ip",
                        })?
                        .parse()
                        .map_err(|e| {
                            StateHandlerError::GenericError(eyre!(
                                "parsing the host's BMC IP address failed: {}",
                                e
                            ))
                        })?;

                    if let Err(ipmitool_error) = ctx
                        .services
                        .ipmi_tool
                        .bmc_cold_reset(
                            bmc_ip_address,
                            &CredentialKey::BmcCredentials {
                                credential_type: BmcCredentialType::BmcRoot { bmc_mac_address },
                            },
                        )
                        .await
                    {
                        tracing::warn!(
                            "Failed to reset BMC for {} through IPMI tool: {ipmitool_error}",
                            &state.host_snapshot.id
                        );

                        return Err(StateHandlerError::GenericError(eyre!(
                            "Failed to reset BMC for {}; redfish error: {redfish_error}; ipmitool error: {ipmitool_error}",
                            &state.host_snapshot.id
                        )));
                    };
                }
            }

            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state_with_all_dpus_updated(state, reprovision_state)?,
            ))
        }
        ReprovisionState::RebootHost => {
            // We can expect transient issues here in case we just rebooted the host's BMC and it has not come up yet
            handler_host_power_control(state, ctx, SystemPowerControl::ForceRestart).await?;

            // We need to wait for the host to reboot and submit its new Hardware information in
            // case of Ready.
            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_state(
                    &state.managed_state,
                    dpu_machine_id,
                    &state.host_snapshot,
                )?,
            ))
        }
        ReprovisionState::NotUnderReprovision => Ok(StateHandlerOutcome::do_nothing()),
    }
}
