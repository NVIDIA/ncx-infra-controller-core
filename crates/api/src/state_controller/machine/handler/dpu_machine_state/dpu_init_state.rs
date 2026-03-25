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

use libredfish::model::oem::nvidia_dpu::HostPrivilegeLevel;
use libredfish::{EnabledDisabled, RedfishError, SystemPowerControl};
use model::machine::{
    DpuInitNextStateResolver, DpuInitState, Machine, MachineState, ManagedHostState,
    ManagedHostStateSnapshot, PerformPowerOperation,
};

use super::super::helpers::{DpuInitStateHelper, ManagedHostStateHelper};
use super::super::host_machine_state::managed_host_network_config_version_synced_and_dpu_healthy;
use super::super::{
    call_machine_setup_and_handle_no_dpu_error, check_fw_component_version, dpf,
    handle_bfb_install_state, handler_host_power_control, handler_restart_dpu,
    trigger_reboot_if_needed, try_wait_for_dpu_discovery, wait,
};
use super::DpuMachineStateHandler;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(super) async fn handle(
    handler: &DpuMachineStateHandler,
    state: &ManagedHostStateSnapshot,
    dpu_snapshot: &Machine,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let dpu_machine_id = &dpu_snapshot.id;
    let dpu_state = match &state.managed_state {
        ManagedHostState::DPUInit { dpu_states } => dpu_states
            .states
            .get(dpu_machine_id)
            .ok_or_else(|| StateHandlerError::MissingData {
                object_id: dpu_machine_id.to_string(),
                missing: "dpu_state",
            })?,
        _ => {
            return Err(StateHandlerError::InvalidState(
                "Unexpected state.".to_string(),
            ));
        }
    };
    match &dpu_state {
        DpuInitState::InstallDpuOs { substate } => {
            handle_bfb_install_state(
                state,
                substate.clone(),
                dpu_snapshot,
                ctx,
                &DpuInitNextStateResolver {},
            )
            .await
        }
        DpuInitState::Init => {
            // initial restart, firmware update and scout is run, first reboot of dpu discovery
            let dpu_discovery_result = try_wait_for_dpu_discovery(
                state,
                &handler.reachability_params,
                ctx,
                false,
                dpu_machine_id,
            )
            .await?;

            if let Some(dpu_id) = dpu_discovery_result {
                return Ok(StateHandlerOutcome::wait(format!(
                    "Waiting for DPU {dpu_id} discovery and reboot"
                )));
            }

            tracing::debug!(
                "ManagedHostState::DPUNotReady::Init: firmware update enabled = {}",
                handler.dpu_nic_firmware_initial_update_enabled
            );

            // All DPUs are discovered. Reboot them to proceed.
            for dpu_snapshot in &state.dpu_snapshots {
                handler_restart_dpu(
                    dpu_snapshot,
                    ctx,
                    state.host_snapshot.dpf.used_for_ingestion,
                )
                .await?;
            }

            let machine_state = DpuInitState::WaitingForPlatformPowercycle {
                substate: PerformPowerOperation::Off,
            };
            let next_state =
                machine_state.next_state_with_all_dpus_updated(&state.managed_state)?;
            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuInitState::DpfStates { state: dpf_state } => {
            let dpf_sdk = handler.dpf_sdk.as_deref().ok_or_else(|| {
                StateHandlerError::GenericError(eyre::eyre!(
                    "DPF state reached but DPF is not configured"
                ))
            })?;
            dpf::handle_dpf_state(state, dpu_snapshot, dpf_state, ctx, dpf_sdk).await
        }
        DpuInitState::WaitingForPlatformPowercycle {
            substate: PerformPowerOperation::Off,
        } => {
            // Wait until all DPUs arrive in Off state.
            if !state.managed_state.all_dpu_states_in_sync()? {
                return Ok(StateHandlerOutcome::wait(
                    "Waiting for all dpus to move to off state.".to_string(),
                ));
            }

            handler_host_power_control(state, ctx, SystemPowerControl::ForceOff).await?;

            let next_state = DpuInitState::WaitingForPlatformPowercycle {
                substate: PerformPowerOperation::On,
            }
            .next_state_with_all_dpus_updated(&state.managed_state)?;

            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuInitState::WaitingForPlatformPowercycle {
            substate: PerformPowerOperation::On,
        } => {
            let basetime = state
                .host_snapshot
                .last_reboot_requested
                .as_ref()
                .map(|x| x.time)
                .unwrap_or(state.host_snapshot.state.version.timestamp());

            if wait(&basetime, handler.reachability_params.power_down_wait) {
                return Ok(StateHandlerOutcome::wait(format!(
                    "Waiting for power_down_wait ({}m) to elapse before powering on host",
                    handler.reachability_params.power_down_wait.num_minutes(),
                )));
            }

            handler_host_power_control(state, ctx, SystemPowerControl::On).await?;

            let next_state = DpuInitState::WaitingForPlatformConfiguration
                .next_state_with_all_dpus_updated(&state.managed_state)?;

            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuInitState::WaitingForPlatformConfiguration => {
            let dpu_redfish_client = match ctx
                .services
                .create_redfish_client_from_machine(dpu_snapshot)
                .await
            {
                Ok(client) => client,
                Err(e) => {
                    let msg = format!(
                        "failed to create redfish client for DPU {}, potentially because we turned the host off as part of error handling in this state. err: {}",
                        dpu_snapshot.id, e
                    );
                    tracing::warn!(msg);
                    // If we cannot create a redfish client for the DPU, this function call will never result in an actual DPU reboot.
                    // The only side effect is turning the DPU's host back on if we turned it off earlier.
                    let reboot_status = trigger_reboot_if_needed(
                        dpu_snapshot,
                        state,
                        None,
                        &handler.reachability_params,
                        ctx,
                    )
                    .await?;

                    return Ok(StateHandlerOutcome::wait(format!(
                        "{msg};\nDPU reboot status: {reboot_status:#?}",
                    )));
                }
            };

            if let Some(outcome) =
                check_fw_component_version(ctx, dpu_snapshot, &handler.hardware_models).await?
            {
                return Ok(outcome);
            }

            let boot_interface_mac = None; // libredfish will choose the DPU
            if handler.enable_secure_boot {
                dpu_redfish_client
                    .set_host_rshim(EnabledDisabled::Disabled)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "set_host_rshim",
                        error: e,
                    })?;
                dpu_redfish_client
                    .set_host_privilege_level(HostPrivilegeLevel::Restricted)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "set_host_privilege_level",
                        error: e,
                    })?;
            } else if let Err(e) = call_machine_setup_and_handle_no_dpu_error(
                dpu_redfish_client.as_ref(),
                boot_interface_mac,
                state.host_snapshot.associated_dpu_machine_ids().len(),
                &ctx.services.site_config,
            )
            .await
            {
                let msg = format!(
                    "redfish machine_setup failed for DPU {}, potentially due to known race condition between UEFI POST and BMC. issuing a force-restart. err: {}",
                    dpu_snapshot.id, e
                );
                tracing::warn!(msg);
                let reboot_status = trigger_reboot_if_needed(
                    dpu_snapshot,
                    state,
                    None,
                    &handler.reachability_params,
                    ctx,
                )
                .await?;

                return Ok(StateHandlerOutcome::wait(format!(
                    "{msg};\nWaiting for DPU {} to reboot: {reboot_status:#?}",
                    dpu_snapshot.id
                )));
            }

            if let Err(e) = ctx
                .services
                .redfish_client_pool
                .uefi_setup(dpu_redfish_client.as_ref(), true)
                .await
            {
                let msg = format!(
                    "Failed to run uefi_setup call failed for DPU {}: {}",
                    dpu_snapshot.id, e
                );
                tracing::warn!(msg);
                let reboot_status = trigger_reboot_if_needed(
                    dpu_snapshot,
                    state,
                    None,
                    &handler.reachability_params,
                    ctx,
                )
                .await?;

                return Ok(StateHandlerOutcome::wait(format!(
                    "{msg};\nWaiting for DPU {} to reboot: {reboot_status:#?}",
                    dpu_snapshot.id
                )));
            }

            // We need to reboot the DPU after configuring the BIOS settings appropriately
            // so that they are applied
            handler_restart_dpu(
                dpu_snapshot,
                ctx,
                state.host_snapshot.dpf.used_for_ingestion,
            )
            .await?;

            let next_state =
                DpuInitState::PollingBiosSetup.next_state(&state.managed_state, dpu_machine_id)?;

            Ok(StateHandlerOutcome::transition(next_state))
        }

        DpuInitState::PollingBiosSetup => {
            let next_state = DpuInitState::WaitingForNetworkConfig
                .next_state(&state.managed_state, dpu_machine_id)?;

            let dpu_redfish_client = match ctx
                .services
                .create_redfish_client_from_machine(dpu_snapshot)
                .await
            {
                Ok(client) => client,
                Err(e) => {
                    return Err(StateHandlerError::RedfishError {
                        operation: "create_client_from_machine",
                        error: RedfishError::GenericError {
                            error: e.to_string(),
                        },
                    });
                }
            };

            match dpu_redfish_client.is_bios_setup(None).await {
                Ok(true) => {
                    tracing::info!(
                        dpu_id = %dpu_snapshot.id,
                        "BIOS setup verified successfully for DPU"
                    );
                    Ok(StateHandlerOutcome::transition(next_state))
                }
                Ok(false) => Ok(StateHandlerOutcome::wait(format!(
                    "Polling BIOS setup status, waiting for settings to be applied on DPU {}",
                    dpu_snapshot.id
                ))),
                Err(e) => {
                    tracing::warn!(
                        dpu_id = %dpu_snapshot.id,
                        error = %e,
                        "Failed to check DPU BIOS setup status, will retry"
                    );
                    Ok(StateHandlerOutcome::wait(format!(
                        "Failed to check BIOS setup status for DPU {}: {}. Will retry.",
                        dpu_snapshot.id, e
                    )))
                }
            }
        }

        DpuInitState::WaitingForNetworkConfig => {
            // is_network_ready is syncing over all DPUs.
            // The code will move only when all DPUs returns network_ready signal.
            for dsnapshot in &state.dpu_snapshots {
                if !managed_host_network_config_version_synced_and_dpu_healthy(dsnapshot) {
                    let mut reboot_status = None;
                    // Only reboot the DPU which is targeted in this event loop.
                    if dsnapshot.id == dpu_snapshot.id {
                        // we requested a DPU reboot in DpuInitState::Init
                        // let the trigger_reboot_if_needed determine if we are stuck here
                        // (based on how long it has been since the last requested reboot)
                        reboot_status = Some(
                            trigger_reboot_if_needed(
                                dsnapshot,
                                state,
                                None,
                                &handler.reachability_params,
                                ctx,
                            )
                            .await?,
                        );
                    }

                    // TODO: Make is_network_ready give us more details as a string
                    return Ok(StateHandlerOutcome::wait(format!(
                        "Waiting for DPU agent to apply network config and report healthy network for DPU {}\nreboot status: {reboot_status:#?}",
                        dsnapshot.id
                    )));
                }
            }

            let next_state = ManagedHostState::HostInit {
                machine_state: MachineState::EnableIpmiOverLan,
            };
            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuInitState::WaitingForNetworkInstall => {
            tracing::warn!(
                "Invalid State WaitingForNetworkInstall for dpu Machine {}",
                dpu_machine_id
            );
            Err(StateHandlerError::InvalidHostState(
                *dpu_machine_id,
                Box::new(state.managed_state.clone()),
            ))
        }
    }
}
