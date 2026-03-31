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
use librms::protos::rack_manager::{NewNodeInfo, NodeType as RmsNodeType};
use model::machine::{
    DpuDiscoveringState, DpuDiscoveringStates, ManagedHostState, ManagedHostStateSnapshot,
};

use crate::site_explorer::rms;
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

    // We only reach RegisterRmsMembership via VerifyRmsMembership,
    // which already confirmed rms_client and rack_id exist. But
    // guard defensively in case this state is entered directly
    // (e.g. after a restart with persisted state).
    let Some(rms_client) = &ctx.services.rms_client else {
        tracing::debug!(
            machine_id = %host_machine_id,
            "No RMS client configured, skipping RMS registration"
        );
        return Ok(StateHandlerOutcome::transition(next_state));
    };

    let bmc_mac = mh_snapshot.host_snapshot.bmc_info.mac;
    let expected_machine = if let Some(mac) = bmc_mac {
        db::expected_machine::find_by_bmc_mac_address(&mut ctx.services.db_reader, mac).await?
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
    let Some(rack_id) = rack_id else {
        tracing::debug!(
            machine_id = %host_machine_id,
            "No rack_id configured for machine, skipping RMS registration"
        );
        return Ok(StateHandlerOutcome::transition(next_state));
    };

    let bmc_ip = mh_snapshot
        .host_snapshot
        .bmc_info
        .ip
        .clone()
        .unwrap_or_default();

    // TODO(chet): If a node already exists, it returns some sense
    // of an "already exists" error. However, the proto spec doesn't
    // seem to define this, so once that's sorted, make sure to
    // integrate that here.
    let new_node_info = NewNodeInfo {
        rack_id: rack_id.to_string(),
        node_id: host_machine_id.to_string(),
        mac_address: bmc_mac.unwrap_or_default().to_string(),
        ip_address: bmc_ip,
        port: 443,
        username: None,
        password: None,
        r#type: Some(RmsNodeType::Compute.into()),
        vault_path: String::new(),
        host_ip_addresses: vec![],
        host_mac_addresses: vec![],
    };
    match rms::add_node_to_rms(rms_client.as_ref(), new_node_info).await {
        Ok(()) => {
            tracing::info!(
                machine_id = %host_machine_id,
                "Successfully registered machine with RMS"
            );
            // NOTE: We could also transition back to VerifyRmsMembership
            // here to confirm the registration took effect. For now, we
            // trust that a successful add_node means we're registered
            // and move forward.
            Ok(StateHandlerOutcome::transition(next_state))
        }
        Err(e) => {
            tracing::warn!(
                machine_id = %host_machine_id,
                "Failed to register machine with RMS: {e}, will retry"
            );
            Ok(StateHandlerOutcome::wait(
                "Waiting to retry RMS registration".to_string(),
            ))
        }
    }
}
