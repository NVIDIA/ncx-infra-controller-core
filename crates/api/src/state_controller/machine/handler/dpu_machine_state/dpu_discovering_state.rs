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

use libredfish::Boot;
use model::machine::{
    DpuDiscoveringState, DpuInitState, Machine, ManagedHostState, ManagedHostStateSnapshot,
    NextStateBFBSupport, SetSecureBootState, dpf_based_dpu_provisioning_possible,
};

use super::super::handler_restart_dpu;
use super::super::helpers::{
    DpuDiscoveringStateHelper, DpuInitStateHelper, ManagedHostStateHelper,
};
use super::DpuMachineStateHandler;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle(
    handler: &DpuMachineStateHandler,
    state: &ManagedHostStateSnapshot,
    dpu_snapshot: &Machine,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let dpu_machine_id = &dpu_snapshot.id.clone();
    let current_dpu_state = match &state.managed_state {
        ManagedHostState::DpuDiscoveringState { dpu_states } => dpu_states
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

    let dpu_redfish_client_result = ctx
        .services
        .create_redfish_client_from_machine(dpu_snapshot)
        .await;

    let dpu_redfish_client = match dpu_redfish_client_result {
        Ok(redfish_client) => redfish_client,
        Err(e) => {
            return Ok(StateHandlerOutcome::wait(format!(
                "Waiting for RedFish to become available: {:?}",
                e
            )));
        }
    };

    match current_dpu_state {
        DpuDiscoveringState::Initializing => {
            let next_state = DpuDiscoveringState::Configuring
                .next_state(&state.managed_state, dpu_machine_id)?;
            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuDiscoveringState::Configuring => {
            let next_state = DpuDiscoveringState::EnableRshim
                .next_state(&state.managed_state, dpu_machine_id)?;
            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuDiscoveringState::EnableRshim => {
            let _ = dpu_redfish_client
                .enable_rshim_bmc()
                .await
                .map_err(|e| tracing::info!("failed to enable rshim on DPU {e}"));

            let next_dpu_discovering_state =
                DpuDiscoveringState::next_substate_based_on_bfb_support(
                    handler.enable_secure_boot,
                    state,
                    ctx.services.site_config.dpf.enabled,
                );

            tracing::info!(
                "DPU {dpu_machine_id} (BMC FW version: {}); next_state: {}.",
                dpu_snapshot
                    .bmc_info
                    .firmware_version
                    .clone()
                    .unwrap_or("unknown".to_string()),
                next_dpu_discovering_state
            );

            let next_state =
                next_dpu_discovering_state.next_state(&state.managed_state, dpu_machine_id)?;
            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuDiscoveringState::EnableSecureBoot {
            count,
            enable_secure_boot_state,
            ..
        } => {
            handler
                .set_secure_boot(
                    *count,
                    state,
                    enable_secure_boot_state.clone(),
                    true,
                    dpu_snapshot,
                    dpu_redfish_client.as_ref(),
                )
                .await
        }
        // The proceure to disable secure boot is documented on page 58-59 here: https://docs.nvidia.com/networking/display/nvidia-bluefield-management-and-initial-provisioning.pdf
        DpuDiscoveringState::DisableSecureBoot {
            disable_secure_boot_state,
            count,
        } => {
            handler
                .set_secure_boot(
                    *count,
                    state,
                    disable_secure_boot_state
                        .clone()
                        .unwrap_or(SetSecureBootState::CheckSecureBootStatus),
                    false,
                    dpu_snapshot,
                    dpu_redfish_client.as_ref(),
                )
                .await
        }

        DpuDiscoveringState::SetUefiHttpBoot => {
            // This configures the DPU to boot once from UEFI HTTP.
            //
            // NOTE: since we don't have interface names yet (see comment about UEFI not
            // guaranteed to have POSTed), it will loop through all the interfaces between
            // IPv4, IPv6 so it may take a while.
            //
            dpu_redfish_client
                .boot_once(Boot::UefiHttp)
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "boot_once",
                    error: e,
                })?;

            let next_state = DpuDiscoveringState::RebootAllDPUS
                .next_state(&state.managed_state, dpu_machine_id)?;
            Ok(StateHandlerOutcome::transition(next_state))
        }
        DpuDiscoveringState::RebootAllDPUS => {
            if !state.managed_state.all_dpu_states_in_sync()? {
                return Ok(StateHandlerOutcome::wait(
                    "Waiting for all dpus to finish configuring.".to_string(),
                ));
            }

            if dpf_based_dpu_provisioning_possible(state, handler.dpf_sdk.is_some(), false) {
                let mut txn = ctx.services.db_pool.begin().await?;
                db::machine::mark_machine_ingestion_done_with_dpf(
                    &mut txn,
                    &state.host_snapshot.id,
                )
                .await?;

                let next_state = DpuInitState::DpfStates {
                    state: model::machine::DpfState::Provisioning,
                }
                .next_state_with_all_dpus_updated(&state.managed_state)?;

                return Ok(StateHandlerOutcome::transition(next_state).with_txn(txn));
            }

            for dpu_snapshot in &state.dpu_snapshots {
                handler_restart_dpu(
                    dpu_snapshot,
                    ctx,
                    state.host_snapshot.dpf.used_for_ingestion,
                )
                .await?;
            }
            let next_state =
                DpuInitState::Init.next_state_with_all_dpus_updated(&state.managed_state)?;
            Ok(StateHandlerOutcome::transition(next_state))
        }
    }
}
