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

mod helpers;
mod instance_state_dispatch;

use std::sync::Arc;

use carbide_uuid::machine::MachineId;
pub use helpers::release_vpc_dpu_loopback;
use model::machine::{ManagedHostState, ManagedHostStateSnapshot};
use model::resource_pool::common::CommonPools;

use super::ReachabilityParams;
use super::host_reprovision_state::HostUpgradeState;
use crate::cfg::file::FirmwareConfig;
use crate::dpf::DpfOperations;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandler, StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

/// A `StateHandler` implementation for instances
#[derive(Debug, Clone)]
pub struct InstanceStateHandler {
    attestation_enabled: bool,
    reachability_params: ReachabilityParams,
    common_pools: Option<Arc<CommonPools>>,
    host_upgrade: Arc<HostUpgradeState>,
    hardware_models: FirmwareConfig,
    enable_secure_boot: bool,
    dpf_sdk: Option<Arc<dyn DpfOperations>>,
}

impl InstanceStateHandler {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        attestation_enabled: bool,
        reachability_params: ReachabilityParams,
        common_pools: Option<Arc<CommonPools>>,
        host_upgrade: Arc<HostUpgradeState>,
        hardware_models: FirmwareConfig,
        enable_secure_boot: bool,
        dpf_sdk: Option<Arc<dyn DpfOperations>>,
    ) -> Self {
        InstanceStateHandler {
            attestation_enabled,
            reachability_params,
            common_pools,
            host_upgrade,
            hardware_models,
            enable_secure_boot,
            dpf_sdk,
        }
    }
}

#[async_trait::async_trait]
impl StateHandler for InstanceStateHandler {
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
        instance_state_dispatch::handle(self, host_machine_id, mh_snapshot, ctx).await
    }
}
