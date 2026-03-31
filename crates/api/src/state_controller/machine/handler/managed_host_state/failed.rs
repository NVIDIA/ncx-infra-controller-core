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

use std::mem::discriminant as enum_discr;

use carbide_uuid::machine::MachineId;
use db::db_read::PgPoolReader;
use libredfish::SystemPowerControl;
use measured_boot::records::MeasurementMachineState;
use model::machine::{
    CleanupState, FailureCause, FailureDetails, FailureSource, LockdownInfo, LockdownMode,
    LockdownState, MachineState, ManagedHostState, ManagedHostStateSnapshot, MeasuringState,
    StateMachineArea,
};

use super::super::machine_validation::handle_machine_validation_requested;
use super::super::{
    ReachabilityParams, cleanedup_after_state_transition, discovered_after_state_transition,
    handler_host_power_control, trigger_reboot_if_needed,
};
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::get_measuring_prerequisites;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle(
    host_machine_id: &MachineId,
    mh_snapshot: &mut ManagedHostStateSnapshot,
    details: &FailureDetails,
    machine_id: &MachineId,
    retry_count: u32,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    attestation_enabled: bool,
    reachability_params: &ReachabilityParams,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    match details.cause {
        // DPU discovery failed needs more logic to handle.
        // DPU discovery can failed from multiple states init,
        // waitingfornetworkinstall, reprov(waitingforfirmwareupgrade),
        // reprov(waitingfornetworkinstall). Error handler must be aware of it and
        // handle based on it.
        // Another bigger problem is every discovery will need a
        // fresh os install as scout is executed by cloud-init and it runs only
        // once after os install. This has to be changed.
        FailureCause::Discovery { .. } if machine_id.machine_type().is_host() => {
            // If user manually reboots host, and discovery is successful then also it will come out
            // of failed state.
            if discovered_after_state_transition(
                mh_snapshot.host_snapshot.state.version,
                mh_snapshot.host_snapshot.last_discovery_time,
            ) {
                ctx.metrics
                    .machine_reboot_attempts_in_failed_during_discovery = Some(retry_count as u64);
                // Anytime host discovery is successful, move to next state.
                let mut txn = ctx.services.db_pool.begin().await?;
                db::machine::clear_failure_details(machine_id, &mut txn).await?;
                let next_state = ManagedHostState::HostInit {
                    machine_state: MachineState::WaitingForLockdown {
                        lockdown_info: LockdownInfo {
                            state: LockdownState::SetLockdown,
                            mode: LockdownMode::Enable,
                        },
                    },
                };
                return Ok(StateHandlerOutcome::transition(next_state).with_txn(txn));
            }

            // Wait till failure_retry_time is over except first time.
            // First time, host is already up and reported that discovery is failed.
            // Let's reboot now immediately.
            if retry_count == 0 {
                handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart)
                    .await?;
                let next_state = ManagedHostState::Failed {
                    retry_count: retry_count + 1,
                    details: details.clone(),
                    machine_id: *machine_id,
                };
                return Ok(StateHandlerOutcome::transition(next_state));
            }

            if trigger_reboot_if_needed(
                &mh_snapshot.host_snapshot,
                mh_snapshot,
                Some(retry_count as i64),
                reachability_params,
                ctx,
            )
            .await?
            .increase_retry_count
            {
                let next_state = ManagedHostState::Failed {
                    retry_count: retry_count + 1,
                    details: details.clone(),
                    machine_id: *machine_id,
                };
                Ok(StateHandlerOutcome::transition(next_state))
            } else {
                Ok(StateHandlerOutcome::do_nothing())
            }
        }
        FailureCause::NVMECleanFailed { .. } if machine_id.machine_type().is_host() => {
            if cleanedup_after_state_transition(
                mh_snapshot.host_snapshot.state.version,
                mh_snapshot.host_snapshot.last_cleanup_time,
            ) && mh_snapshot.host_snapshot.failure_details.failed_at
                < mh_snapshot
                    .host_snapshot
                    .last_cleanup_time
                    .unwrap_or_default()
            {
                // Cleaned up successfully after a failure.
                let next_state = ManagedHostState::WaitingForCleanup {
                    cleanup_state: CleanupState::Init,
                };
                let mut txn = ctx.services.db_pool.begin().await?;
                db::machine::clear_failure_details(machine_id, &mut txn).await?;
                return Ok(StateHandlerOutcome::transition(next_state).with_txn(txn));
            }

            if trigger_reboot_if_needed(
                &mh_snapshot.host_snapshot,
                mh_snapshot,
                Some(retry_count as i64),
                reachability_params,
                ctx,
            )
            .await?
            .increase_retry_count
            {
                let next_state = ManagedHostState::Failed {
                    retry_count: retry_count + 1,
                    details: details.clone(),
                    machine_id: *machine_id,
                };
                Ok(StateHandlerOutcome::transition(next_state))
            } else {
                Ok(StateHandlerOutcome::do_nothing())
            }
        }
        FailureCause::MeasurementsRetired { .. }
        | FailureCause::MeasurementsRevoked { .. }
        | FailureCause::MeasurementsCAValidationFailed { .. } => {
            if check_if_not_in_original_failure_cause_anymore(
                &mh_snapshot.host_snapshot.id,
                &mut ctx.services.db_reader,
                &details.cause,
                attestation_enabled,
            )
            .await?
            {
                // depending on the source of the failure, move it to the correct measuring state
                match &details.source {
                        FailureSource::StateMachineArea(area) => {
                            match area{
                                StateMachineArea::MainFlow => Ok(StateHandlerOutcome::transition(
                                    ManagedHostState::Measuring {
                                        measuring_state: MeasuringState::WaitingForMeasurements
                                    })),
                                StateMachineArea::HostInit => Ok(StateHandlerOutcome::transition(
                                    ManagedHostState::HostInit {
                                        machine_state: MachineState::Measuring{
                                            measuring_state: MeasuringState::WaitingForMeasurements
                                        }
                                    })),
                                StateMachineArea::AssignedInstance => Ok(StateHandlerOutcome::transition(
                                    ManagedHostState::PostAssignedMeasuring {
                                            measuring_state: MeasuringState::WaitingForMeasurements
                                    })),
                                _ => Err(StateHandlerError::InvalidState(
                                    "Unimplemented StateMachineArea for FailureSource of  MeasurementsRetired, MeasurementsRevoked, MeasurementsCAValidationFailed"
                                        .to_string(),
                                ))
                            }
                        },
                        _ => Err(StateHandlerError::InvalidState(
                            "The source of MeasurementsRetired, MeasurementsRevoked, MeasurementsCAValidationFailed can only be StateMachine"
                                .to_string(),
                        ))
                    }
            } else {
                Ok(StateHandlerOutcome::do_nothing())
            }
        }
        FailureCause::MachineValidation { .. } if machine_id.machine_type().is_host() => {
            match handle_machine_validation_requested(ctx.services, mh_snapshot, true).await? {
                Some(outcome) => Ok(outcome),
                None => Ok(StateHandlerOutcome::do_nothing()),
            }
        }
        _ => {
            // Do nothing.
            // Handle error cause and decide how to recover if possible.
            tracing::error!(
                %machine_id,
                "ManagedHost {} is in Failed state with machine/cause {}/{}. Failed at: {}, Ignoring.",
                host_machine_id,
                machine_id,
                details.cause,
                details.failed_at,
            );
            // TODO: Should this be StateHandlerError::ManualInterventionRequired ?
            Ok(StateHandlerOutcome::do_nothing())
        }
    }
}

pub(crate) async fn check_if_should_redo_measurements(
    machine_id: &MachineId,
    txn: &mut PgPoolReader,
) -> Result<bool, StateHandlerError> {
    let (machine_state, ek_cert_verification_status) =
        get_measuring_prerequisites(machine_id, txn).await?;

    if !ek_cert_verification_status.signing_ca_found {
        return Ok(true);
    }
    match machine_state {
        MeasurementMachineState::Measured => Ok(false),
        _ => Ok(true),
    }
}

async fn check_if_not_in_original_failure_cause_anymore(
    machine_id: &MachineId,
    txn: &mut PgPoolReader,
    original_failure_cause: &FailureCause,
    attestation_enabled: bool,
) -> Result<bool, StateHandlerError> {
    if !attestation_enabled {
        return Ok(true);
    }
    let (_, ek_cert_verification_status) = get_measuring_prerequisites(machine_id, txn).await?;

    // if the failure cause was ca validation and it no longer is, then we can try
    // transitioning to the Measuring state to see where that takes us further
    if enum_discr(original_failure_cause)
        == enum_discr(&FailureCause::MeasurementsCAValidationFailed {
            err: "Dummy error".to_string(),
        })
        && ek_cert_verification_status.signing_ca_found
    {
        return Ok(true);
    }

    let current_failure_cause =
        crate::state_controller::machine::get_measurement_failure_cause(txn, machine_id).await;

    if let Ok(current_failure_cause) = current_failure_cause {
        match original_failure_cause {
            FailureCause::MeasurementsRetired { .. } => {
                // if current/latest failure cause is the same
                // do nothing
                if enum_discr(&current_failure_cause)
                    == enum_discr(&FailureCause::MeasurementsRetired {
                        err: "Dummy error".to_string(),
                    })
                {
                    Ok(false) // nothing has changed
                } else {
                    Ok(true) // the state has changed
                }
            }
            FailureCause::MeasurementsRevoked { .. } => {
                // if current/latest failure cause is the same
                // do nothing
                if enum_discr(&current_failure_cause)
                    == enum_discr(&FailureCause::MeasurementsRevoked {
                        err: "Dummy error".to_string(),
                    })
                {
                    Ok(false) // nothing has changed
                } else {
                    Ok(true) // the state has changed
                }
            }
            FailureCause::MeasurementsCAValidationFailed { .. } => {
                if ek_cert_verification_status.signing_ca_found {
                    Ok(true) // it has changed
                } else {
                    Ok(false) // nothing has changed
                }
            }
            _ => Ok(true), // it has definitely changed (although we shouldn't be here)
        }
    } else {
        Ok(true) // something has definitely changed
    }
}
