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
use model::machine::{
    CleanupState, FailureDetails, FailureSource, ManagedHostState, MeasuringState, StateMachineArea,
};

use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::{MeasuringOutcome, handle_measuring_state};
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle(
    measuring_state: &MeasuringState,
    host_machine_id: &MachineId,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    attestation_enabled: bool,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    handle_measuring_state(
        measuring_state,
        host_machine_id,
        &mut ctx.services.db_reader,
        attestation_enabled,
    )
    .await
    .map(|v| map_post_assigned_measuring_outcome_to_state_handler_outcome(&v, measuring_state))?
}

pub(crate) fn map_post_assigned_measuring_outcome_to_state_handler_outcome(
    measuring_outcome: &MeasuringOutcome,
    measuring_state: &MeasuringState,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    match measuring_outcome {
        MeasuringOutcome::NoChange => Ok(StateHandlerOutcome::wait(
            match measuring_state {
                MeasuringState::WaitingForMeasurements => {
                    "Waiting for machine to send measurement report"
                }
                MeasuringState::PendingBundle => {
                    "Waiting for matching measurement bundle for machine profile"
                }
            }
            .to_string(),
        )),
        MeasuringOutcome::WaitForGoldenValues => Ok(StateHandlerOutcome::transition(
            ManagedHostState::PostAssignedMeasuring {
                measuring_state: MeasuringState::PendingBundle,
            },
        )),
        MeasuringOutcome::WaitForScoutToSendMeasurements => Ok(StateHandlerOutcome::transition(
            ManagedHostState::PostAssignedMeasuring {
                measuring_state: MeasuringState::WaitingForMeasurements,
            },
        )),
        MeasuringOutcome::Unsuccessful((failure_details, machine_id)) => {
            Ok(StateHandlerOutcome::transition(ManagedHostState::Failed {
                details: FailureDetails {
                    cause: failure_details.cause.clone(),
                    failed_at: failure_details.failed_at,
                    source: FailureSource::StateMachineArea(StateMachineArea::AssignedInstance),
                },
                machine_id: *machine_id,
                retry_count: 0,
            }))
        }
        MeasuringOutcome::PassedOk => Ok(StateHandlerOutcome::transition(
            ManagedHostState::WaitingForCleanup {
                cleanup_state: CleanupState::Init,
            },
        )),
    }
}
