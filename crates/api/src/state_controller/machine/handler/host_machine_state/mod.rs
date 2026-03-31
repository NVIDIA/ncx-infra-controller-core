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

mod handle_host_boot_order_setup;
mod handle_host_uefi_setup;
mod machine_state;

mod handle_instance_host_platform_config;

use carbide_uuid::machine::MachineId;
pub(super) use handle_instance_host_platform_config::handle_instance_host_platform_config;
use model::machine::{Machine, ManagedHostState, ManagedHostStateSnapshot};

use super::HostHandlerParams;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandler, StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

/// A `StateHandler` implementation for host machines
#[derive(Debug, Clone)]
pub struct HostMachineStateHandler {
    pub(super) host_handler_params: HostHandlerParams,
}

impl HostMachineStateHandler {
    pub fn new(host_handler_params: HostHandlerParams) -> Self {
        Self {
            host_handler_params,
        }
    }
}

pub(super) fn managed_host_network_config_version_synced_and_dpu_healthy(
    dpu_snapshot: &Machine,
) -> bool {
    if !dpu_snapshot.managed_host_network_config_version_synced() {
        return false;
    }

    let Some(dpu_health) = &dpu_snapshot.dpu_agent_health_report else {
        return false;
    };

    // Note that DPU alerts may be surpressed (classifications removed) in the aggregate health
    // report so the individual DPU's report is used.
    !dpu_health
        .has_classification(&health_report::HealthAlertClassification::prevent_host_state_changes())
}

#[async_trait::async_trait]
impl StateHandler for HostMachineStateHandler {
    type State = ManagedHostStateSnapshot;
    type ControllerState = ManagedHostState;
    type ObjectId = MachineId;
    type ContextObjects = MachineStateHandlerContextObjects;

    async fn handle_object_state(
        &self,
        host_machine_id: &MachineId,
        mh_snapshot: &mut ManagedHostStateSnapshot,
        _controller_state: &Self::ControllerState,
        ctx: &mut StateHandlerContext<Self::ContextObjects>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        if let ManagedHostState::HostInit { machine_state } = mh_snapshot.managed_state.clone() {
            machine_state::handle(
                host_machine_id,
                mh_snapshot,
                ctx,
                &self.host_handler_params,
                &machine_state,
            )
            .await
        } else {
            Err(StateHandlerError::InvalidHostState(
                *host_machine_id,
                Box::new(mh_snapshot.managed_state.clone()),
            ))
        }
    }
}
