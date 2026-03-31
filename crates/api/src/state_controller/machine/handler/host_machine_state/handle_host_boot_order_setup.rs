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

use model::machine::{
    MachineState, ManagedHostState, ManagedHostStateSnapshot, MeasuringState, SetBootOrderInfo,
    SetBootOrderState,
};

use super::super::{HostHandlerParams, SetBootOrderOutcome, set_host_boot_order};
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle_host_boot_order_setup(
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    host_handler_params: HostHandlerParams,
    mh_snapshot: &mut ManagedHostStateSnapshot,
    set_boot_order_info: Option<SetBootOrderInfo>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    tracing::info!(
        "Starting Boot Order Configuration for {}: {set_boot_order_info:#?}",
        mh_snapshot.host_snapshot.id
    );

    let redfish_client = ctx
        .services
        .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
        .await?;

    let next_state = match set_boot_order_info {
        Some(info) => {
            match set_host_boot_order(
                ctx,
                &host_handler_params.reachability_params,
                redfish_client.as_ref(),
                mh_snapshot,
                info,
            )
            .await?
            {
                SetBootOrderOutcome::Continue(boot_order_info) => ManagedHostState::HostInit {
                    machine_state: MachineState::SetBootOrder {
                        set_boot_order_info: Some(boot_order_info),
                    },
                },
                SetBootOrderOutcome::Done => {
                    if host_handler_params.attestation_enabled {
                        ManagedHostState::HostInit {
                            machine_state: MachineState::Measuring {
                                measuring_state: MeasuringState::WaitingForMeasurements,
                            },
                        }
                    } else {
                        ManagedHostState::HostInit {
                            machine_state: MachineState::WaitingForDiscovery,
                        }
                    }
                }
                SetBootOrderOutcome::WaitingForReboot(reason) => {
                    return Ok(StateHandlerOutcome::wait(reason));
                }
            }
        }
        None => ManagedHostState::HostInit {
            machine_state: MachineState::SetBootOrder {
                set_boot_order_info: Some(SetBootOrderInfo {
                    set_boot_order_jid: None,
                    set_boot_order_state: SetBootOrderState::SetBootOrder,
                    retry_count: 0,
                }),
            },
        },
    };

    Ok(StateHandlerOutcome::transition(next_state))
}
