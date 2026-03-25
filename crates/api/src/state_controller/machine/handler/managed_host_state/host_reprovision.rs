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
use model::machine::{ManagedHostState, ManagedHostStateSnapshot};

use super::super::HostUpgradeState;
use super::super::host_reprovision_state::HostFirmwareScenario;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle(
    host_upgrade: &HostUpgradeState,
    mh_snapshot: &mut ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    host_machine_id: &MachineId,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    host_upgrade
        .handle_host_reprovision(
            mh_snapshot,
            ctx,
            host_machine_id,
            HostFirmwareScenario::Ready,
        )
        .await
}
