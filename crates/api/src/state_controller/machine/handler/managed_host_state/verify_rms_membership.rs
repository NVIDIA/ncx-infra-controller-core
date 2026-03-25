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
use librms::RackManagerError;
use model::machine::{
    DpuDiscoveringState, DpuDiscoveringStates, ManagedHostState, ManagedHostStateSnapshot,
};

use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle(
    host_machine_id: &MachineId,
    mh_snapshot: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    _enable_secure_boot: bool,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let dpu_ids = mh_snapshot.host_snapshot.associated_dpu_machine_ids();
    let next_state = ManagedHostState::DpuDiscoveringState {
        dpu_states: DpuDiscoveringStates {
            states: dpu_ids
                .into_iter()
                .map(|id| (id, DpuDiscoveringState::Initializing))
                .collect(),
        },
    };

    let Some(rms_client) = &ctx.services.rms_client else {
        tracing::debug!(
            machine_id = %host_machine_id,
            "No RMS client configured, skipping RMS verification"
        );
        return Ok(StateHandlerOutcome::transition(next_state));
    };

    // If there's no rack_id, this machine isn't rack-managed,
    // so skip RMS verification entirely.
    let expected_machine = if let Some(bmc_mac) = mh_snapshot.host_snapshot.bmc_info.mac {
        db::expected_machine::find_by_bmc_mac_address(&mut ctx.services.db_reader, bmc_mac).await?
    } else {
        None
    };

    // TODO(chet): Look into copying the rack_id over to the Machine entry in
    // site-explorer machine creation, which then allows us to skip over the
    // ExpectedMachine lookup. Conceptually, EM == Inventory/Manifest, and the
    // MH == what we're actually managing.
    let rack_id = expected_machine
        .as_ref()
        .and_then(|em| em.data.rack_id.clone());
    if rack_id.is_none() {
        tracing::debug!(
            machine_id = %host_machine_id,
            "No rack_id configured for machine, skipping RMS verification"
        );
        return Ok(StateHandlerOutcome::transition(next_state));
    }

    let node_id_str = host_machine_id.to_string();

    match rms_client
        .get_all_inventory(librms::protos::rack_manager::GetAllInventoryRequest::default())
        .await
    {
        Ok(response) => {
            let node_found = response.nodes.iter().any(|n| n.node_id == node_id_str);
            if node_found {
                tracing::info!(
                    machine_id = %host_machine_id,
                    "Verified machine is registered with RMS, skipping registration"
                );
                Ok(StateHandlerOutcome::transition(next_state))
            } else {
                tracing::info!(
                    machine_id = %host_machine_id,
                    "Machine not found in RMS inventory, registering"
                );
                Ok(StateHandlerOutcome::transition(
                    ManagedHostState::RegisterRmsMembership,
                ))
            }
        }
        Err(e) => {
            // NotFound means the node definitely isn't registered —
            // move on to registration. Any other error (connectivity,
            // internal, etc.) means we should retry verification.
            let is_not_found = matches!(
                &e,
                RackManagerError::ApiInvocationError(status)
                    if status.code() == tonic::Code::NotFound
            );

            if is_not_found {
                tracing::warn!(
                    machine_id = %host_machine_id,
                    "RMS returned NotFound during verification, registering"
                );
                Ok(StateHandlerOutcome::transition(
                    ManagedHostState::RegisterRmsMembership,
                ))
            } else {
                tracing::warn!(
                    machine_id = %host_machine_id,
                    "Failed to verify RMS membership: {e}, will retry"
                );
                Ok(StateHandlerOutcome::wait(
                    "Waiting to retry RMS verification".to_string(),
                ))
            }
        }
    }
}
