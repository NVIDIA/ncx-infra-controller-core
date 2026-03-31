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
use libredfish::{EnabledDisabled, PowerState, Redfish, SystemPowerControl};
use model::machine::{
    BomValidating, BomValidatingContext, CleanupState, CreateBossVolumeContext,
    CreateBossVolumeState, ManagedHostState, ManagedHostStateSnapshot, SecureEraseBossContext,
    SecureEraseBossState,
};

use super::super::{
    ReachabilityParams, cleanedup_after_state_transition, handler_host_power_control,
    trigger_reboot_if_needed, wait,
};
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle(
    host_machine_id: &MachineId,
    mh_snapshot: &ManagedHostStateSnapshot,
    cleanup_state: &CleanupState,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    reachability_params: &ReachabilityParams,
    _host_handler_params: &super::super::HostHandlerParams,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let redfish_client = ctx
        .services
        .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
        .await?;

    match cleanup_state {
        CleanupState::Init => {
            if mh_snapshot.host_snapshot.bmc_vendor().is_dell()
                && let Some(boss_controller_id) = redfish_client
                    .get_boss_controller()
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "get_boss_controller",
                        error: e,
                    })?
            {
                let next_state: ManagedHostState = ManagedHostState::WaitingForCleanup {
                    cleanup_state: CleanupState::SecureEraseBoss {
                        secure_erase_boss_context: SecureEraseBossContext {
                            boss_controller_id,
                            secure_erase_jid: None,
                            secure_erase_boss_state: SecureEraseBossState::UnlockHost,
                            iteration: Some(0),
                        },
                    },
                };

                return Ok(StateHandlerOutcome::transition(next_state));
            }

            let next_state: ManagedHostState = ManagedHostState::WaitingForCleanup {
                cleanup_state: CleanupState::HostCleanup {
                    boss_controller_id: None,
                },
            };

            Ok(StateHandlerOutcome::transition(next_state))
        }
        CleanupState::SecureEraseBoss {
            secure_erase_boss_context,
        } => {
            let boss_controller_id = secure_erase_boss_context.boss_controller_id.clone();

            match secure_erase_boss_context.secure_erase_boss_state {
                SecureEraseBossState::UnlockHost => {
                    redfish_client
                        .set_idrac_lockdown(EnabledDisabled::Disabled)
                        .await
                        .map_err(|e| StateHandlerError::RedfishError {
                            operation: "set_idrac_lockdown",
                            error: e,
                        })?;

                    let next_state: ManagedHostState = ManagedHostState::WaitingForCleanup {
                        cleanup_state: CleanupState::SecureEraseBoss {
                            secure_erase_boss_context: SecureEraseBossContext {
                                boss_controller_id,
                                secure_erase_jid: None,
                                secure_erase_boss_state: SecureEraseBossState::SecureEraseBoss,
                                iteration: secure_erase_boss_context.iteration,
                            },
                        },
                    };

                    Ok(StateHandlerOutcome::transition(next_state))
                }
                SecureEraseBossState::SecureEraseBoss => {
                    let jid = redfish_client
                        .decommission_storage_controller(
                            &secure_erase_boss_context.boss_controller_id,
                        )
                        .await
                        .map_err(|e| StateHandlerError::RedfishError {
                            operation: "decommission_storage_controller",
                            error: e,
                        })?;

                    let next_state: ManagedHostState = ManagedHostState::WaitingForCleanup {
                        cleanup_state: CleanupState::SecureEraseBoss {
                            secure_erase_boss_context: SecureEraseBossContext {
                                boss_controller_id,
                                secure_erase_jid: jid,
                                secure_erase_boss_state: SecureEraseBossState::WaitForJobCompletion,
                                iteration: secure_erase_boss_context.iteration,
                            },
                        },
                    };

                    Ok(StateHandlerOutcome::transition(next_state))
                }
                SecureEraseBossState::WaitForJobCompletion => {
                    wait_for_boss_controller_job_to_complete(redfish_client.as_ref(), mh_snapshot)
                        .await
                }
                SecureEraseBossState::HandleJobFailure {
                    failure: _,
                    power_state: _,
                } => handle_boss_job_failure(redfish_client.as_ref(), mh_snapshot, ctx).await,
            }
        }
        CleanupState::HostCleanup { boss_controller_id } => {
            if !cleanedup_after_state_transition(
                mh_snapshot.host_snapshot.state.version,
                mh_snapshot.host_snapshot.last_cleanup_time,
            ) {
                let status = trigger_reboot_if_needed(
                    &mh_snapshot.host_snapshot,
                    mh_snapshot,
                    None,
                    reachability_params,
                    ctx,
                )
                .await?;
                return Ok(StateHandlerOutcome::wait(status.status));
            }

            // Reboot host
            handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart).await?;

            let next_state = match boss_controller_id {
                Some(boss_controller_id) => ManagedHostState::WaitingForCleanup {
                    cleanup_state: CleanupState::CreateBossVolume {
                        create_boss_volume_context: CreateBossVolumeContext {
                            boss_controller_id: boss_controller_id.to_string(),
                            create_boss_volume_jid: None,
                            create_boss_volume_state: CreateBossVolumeState::CreateBossVolume,
                            iteration: Some(0),
                        },
                    },
                },
                None => ManagedHostState::BomValidating {
                    bom_validating_state: BomValidating::UpdatingInventory(BomValidatingContext {
                        machine_validation_context: Some("Cleanup".to_string()),
                        ..BomValidatingContext::default()
                    }),
                },
            };

            Ok(StateHandlerOutcome::transition(next_state))
        }
        CleanupState::CreateBossVolume {
            create_boss_volume_context,
        } => {
            let boss_controller_id = create_boss_volume_context.boss_controller_id.clone();
            match create_boss_volume_context.create_boss_volume_state {
                CreateBossVolumeState::CreateBossVolume => {
                    let jid = redfish_client
                        .create_storage_volume(
                            &create_boss_volume_context.boss_controller_id,
                            "VD_0",
                        )
                        .await
                        .map_err(|e| StateHandlerError::RedfishError {
                            operation: "create_storage_volume",
                            error: e,
                        })?;

                    let next_state: ManagedHostState = ManagedHostState::WaitingForCleanup {
                        cleanup_state: CleanupState::CreateBossVolume {
                            create_boss_volume_context: CreateBossVolumeContext {
                                boss_controller_id,
                                create_boss_volume_jid: jid,
                                create_boss_volume_state:
                                    CreateBossVolumeState::WaitForJobScheduled,
                                iteration: create_boss_volume_context.iteration,
                            },
                        },
                    };

                    Ok(StateHandlerOutcome::transition(next_state))
                }
                CreateBossVolumeState::WaitForJobScheduled => {
                    let job_id = create_boss_volume_context
                        .create_boss_volume_jid
                        .clone()
                        .ok_or_else(|| {
                            StateHandlerError::GenericError(eyre::eyre!(
                                "could not find job ID in the Create BOSS Volume Context"
                            ))
                        })?;

                    wait_for_boss_controller_job_to_scheduled(
                        redfish_client.as_ref(),
                        mh_snapshot,
                        boss_controller_id,
                        job_id,
                        create_boss_volume_context.iteration,
                    )
                    .await
                }
                CreateBossVolumeState::RebootHost => {
                    redfish_client
                        .power(SystemPowerControl::ForceRestart)
                        .await
                        .map_err(|e| StateHandlerError::RedfishError {
                            operation: "ForceRestart",
                            error: e,
                        })?;

                    let next_state: ManagedHostState = ManagedHostState::WaitingForCleanup {
                        cleanup_state: CleanupState::CreateBossVolume {
                            create_boss_volume_context: CreateBossVolumeContext {
                                boss_controller_id,
                                create_boss_volume_jid: create_boss_volume_context
                                    .create_boss_volume_jid
                                    .clone(),
                                create_boss_volume_state:
                                    CreateBossVolumeState::WaitForJobCompletion,
                                iteration: create_boss_volume_context.iteration,
                            },
                        },
                    };

                    Ok(StateHandlerOutcome::transition(next_state))
                }
                CreateBossVolumeState::WaitForJobCompletion => {
                    wait_for_boss_controller_job_to_complete(redfish_client.as_ref(), mh_snapshot)
                        .await
                }
                CreateBossVolumeState::LockHost => {
                    redfish_client
                        .set_idrac_lockdown(EnabledDisabled::Enabled)
                        .await
                        .map_err(|e| StateHandlerError::RedfishError {
                            operation: "set_idrac_lockdown",
                            error: e,
                        })?;

                    let next_state: ManagedHostState = ManagedHostState::BomValidating {
                        bom_validating_state: BomValidating::UpdatingInventory(
                            BomValidatingContext {
                                machine_validation_context: Some("Cleanup".to_string()),
                                ..BomValidatingContext::default()
                            },
                        ),
                    };

                    Ok(StateHandlerOutcome::transition(next_state))
                }
                CreateBossVolumeState::HandleJobFailure {
                    failure: _,
                    power_state: _,
                } => handle_boss_job_failure(redfish_client.as_ref(), mh_snapshot, ctx).await,
            }
        }
        CleanupState::DisableBIOSBMCLockdown => {
            tracing::error!(
                machine_id = %host_machine_id,
                "DisableBIOSBMCLockdown state is not implemented. Machine stuck in unimplemented state.",
            );
            Err(StateHandlerError::InvalidHostState(
                *host_machine_id,
                Box::new(mh_snapshot.managed_state.clone()),
            ))
        }
    }
}

fn get_next_state_boss_job_failure(
    mh_snapshot: &ManagedHostStateSnapshot,
) -> Result<(ManagedHostState, PowerState), StateHandlerError> {
    let (next_state, expected_power_state) = match &mh_snapshot.host_snapshot.state.value {
        ManagedHostState::WaitingForCleanup { cleanup_state } => match cleanup_state {
            CleanupState::SecureEraseBoss {
                secure_erase_boss_context,
            } => match &secure_erase_boss_context.secure_erase_boss_state {
                SecureEraseBossState::HandleJobFailure {
                    failure,
                    power_state,
                } => match power_state {
                    libredfish::PowerState::Off => (
                        ManagedHostState::WaitingForCleanup {
                            cleanup_state: CleanupState::SecureEraseBoss {
                                secure_erase_boss_context: SecureEraseBossContext {
                                    boss_controller_id: secure_erase_boss_context
                                        .boss_controller_id
                                        .clone(),
                                    secure_erase_jid: None,
                                    iteration: secure_erase_boss_context.iteration,
                                    secure_erase_boss_state:
                                        SecureEraseBossState::HandleJobFailure {
                                            failure: failure.to_string(),
                                            power_state: libredfish::PowerState::On,
                                        },
                                },
                            },
                        },
                        *power_state,
                    ),
                    libredfish::PowerState::On => (
                        ManagedHostState::WaitingForCleanup {
                            cleanup_state: CleanupState::SecureEraseBoss {
                                secure_erase_boss_context: SecureEraseBossContext {
                                    boss_controller_id: secure_erase_boss_context
                                        .boss_controller_id
                                        .clone(),
                                    secure_erase_jid: None,
                                    iteration: Some(
                                        secure_erase_boss_context.iteration.unwrap_or_default() + 1,
                                    ),
                                    secure_erase_boss_state: SecureEraseBossState::SecureEraseBoss,
                                },
                            },
                        },
                        *power_state,
                    ),
                    _ => {
                        return Err(StateHandlerError::GenericError(eyre::eyre!(
                            "unexpected SecureEraseBossState::HandleJobFailure power_state for {}: {:#?}",
                            mh_snapshot.host_snapshot.id,
                            mh_snapshot.host_snapshot.state,
                        )));
                    }
                },
                _ => {
                    return Err(StateHandlerError::GenericError(eyre::eyre!(
                        "unexpected SecureEraseBossState state for {}: {:#?}",
                        mh_snapshot.host_snapshot.id,
                        mh_snapshot.host_snapshot.state,
                    )));
                }
            },
            CleanupState::CreateBossVolume {
                create_boss_volume_context,
            } => match &create_boss_volume_context.create_boss_volume_state {
                CreateBossVolumeState::HandleJobFailure {
                    failure,
                    power_state,
                } => match power_state {
                    libredfish::PowerState::Off => (
                        ManagedHostState::WaitingForCleanup {
                            cleanup_state: CleanupState::CreateBossVolume {
                                create_boss_volume_context: CreateBossVolumeContext {
                                    boss_controller_id: create_boss_volume_context
                                        .boss_controller_id
                                        .clone(),
                                    create_boss_volume_jid: None,
                                    iteration: create_boss_volume_context.iteration,
                                    create_boss_volume_state:
                                        CreateBossVolumeState::HandleJobFailure {
                                            failure: failure.to_string(),
                                            power_state: libredfish::PowerState::On,
                                        },
                                },
                            },
                        },
                        *power_state,
                    ),
                    libredfish::PowerState::On => (
                        ManagedHostState::WaitingForCleanup {
                            cleanup_state: CleanupState::CreateBossVolume {
                                create_boss_volume_context: CreateBossVolumeContext {
                                    boss_controller_id: create_boss_volume_context
                                        .boss_controller_id
                                        .clone(),
                                    create_boss_volume_jid: None,
                                    iteration: Some(
                                        create_boss_volume_context.iteration.unwrap_or_default()
                                            + 1,
                                    ),
                                    create_boss_volume_state:
                                        CreateBossVolumeState::CreateBossVolume,
                                },
                            },
                        },
                        *power_state,
                    ),
                    _ => {
                        return Err(StateHandlerError::GenericError(eyre::eyre!(
                            "unexpected CreateBossVolumeState::HandleJobFailure power state for {}: {:#?}",
                            mh_snapshot.host_snapshot.id,
                            mh_snapshot.host_snapshot.state,
                        )));
                    }
                },
                _ => {
                    return Err(StateHandlerError::GenericError(eyre::eyre!(
                        "unexpected CreateBossVolume state for {}: {:#?}",
                        mh_snapshot.host_snapshot.id,
                        mh_snapshot.host_snapshot.state,
                    )));
                }
            },
            _ => {
                return Err(StateHandlerError::GenericError(eyre::eyre!(
                    "unexpected WaitingForCleanup state for {}: {:#?}",
                    mh_snapshot.host_snapshot.id,
                    mh_snapshot.host_snapshot.state,
                )));
            }
        },
        _ => {
            return Err(StateHandlerError::GenericError(eyre::eyre!(
                "unexpected host state for {}: {:#?}",
                mh_snapshot.host_snapshot.id,
                mh_snapshot.host_snapshot.state,
            )));
        }
    };
    Ok((next_state, expected_power_state))
}

fn handle_boss_controller_job_error(
    boss_controller_id: String,
    iterations: u32,
    secure_erase_boss_controller: bool,
    err: StateHandlerError,
    time_since_state_change: chrono::TimeDelta,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    // Wait for 5 minutes before declaring a true failure and transition to the error handling state.
    // As we use this function to handle two different kinds of errors (and maybe others in the future),
    // the defensive nature of this check will be broadly helpful to differentiate between transient errors
    // and true failures. Here is one particular edge case:
    // It takes a little time between creating and scheduling the secure erase job.
    // If the state machine queries the BMC for the job's state prior to the job being scheduled,
    // the BMC's job service will return a 404. Wait here for five minutes to ensure
    // that the job is scheduled prior to declaring an error.
    if time_since_state_change.num_minutes() < 5 {
        return Err(err);
    }

    // we have retried this operation too many times, lets wait for manual intervention
    if iterations > 3 {
        let action = match secure_erase_boss_controller {
            true => "secure erase",
            false => "create the R1 volume on",
        };

        return Err(StateHandlerError::GenericError(eyre::eyre!(
            "We have gone through {} iterations of trying to {action} the BOSS controller; Waiting for manual intervention: {err}",
            iterations
        )));
    }

    // failure path
    let cleanup_state = match secure_erase_boss_controller {
        // the job to decomission the boss controller failed--lets retry
        true => CleanupState::SecureEraseBoss {
            secure_erase_boss_context: SecureEraseBossContext {
                boss_controller_id,
                secure_erase_jid: None,
                secure_erase_boss_state: SecureEraseBossState::HandleJobFailure {
                    failure: err.to_string(),
                    power_state: libredfish::PowerState::Off,
                },
                iteration: Some(iterations),
            },
        },
        // the job to crate the R1 Volume on top of the BOSS controller failed--lets retry
        false => CleanupState::CreateBossVolume {
            create_boss_volume_context: CreateBossVolumeContext {
                boss_controller_id,
                create_boss_volume_jid: None,
                create_boss_volume_state: CreateBossVolumeState::HandleJobFailure {
                    failure: err.to_string(),
                    power_state: libredfish::PowerState::Off,
                },
                iteration: Some(iterations),
            },
        },
    };

    let next_state: ManagedHostState = ManagedHostState::WaitingForCleanup { cleanup_state };

    Ok(StateHandlerOutcome::transition(next_state))
}

async fn wait_for_boss_controller_job_to_scheduled(
    redfish_client: &dyn Redfish,
    mh_snapshot: &ManagedHostStateSnapshot,
    boss_controller_id: String,
    job_id: String,
    iteration: Option<u32>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let job_state = match redfish_client.get_job_state(&job_id).await {
        Ok(state) => state,
        Err(e) => {
            return handle_boss_controller_job_error(
                boss_controller_id,
                iteration.unwrap_or_default(),
                false,
                StateHandlerError::RedfishError {
                    operation: "get_job_state",
                    error: e,
                },
                mh_snapshot.host_snapshot.state.version.since_state_change(),
            );
        }
    };

    let next_state = match job_state {
        libredfish::JobState::Scheduled => ManagedHostState::WaitingForCleanup {
            cleanup_state: CleanupState::CreateBossVolume {
                create_boss_volume_context: CreateBossVolumeContext {
                    boss_controller_id,
                    create_boss_volume_jid: Some(job_id),
                    create_boss_volume_state: CreateBossVolumeState::RebootHost,
                    iteration,
                },
            },
        },
        libredfish::JobState::Completed => {
            tracing::warn!(
                "CreateBossVolume: job {} for {} completed before being scheduled, skipping reboot",
                job_id,
                mh_snapshot.host_snapshot.id,
            );

            ManagedHostState::WaitingForCleanup {
                cleanup_state: CleanupState::CreateBossVolume {
                    create_boss_volume_context: CreateBossVolumeContext {
                        boss_controller_id,
                        create_boss_volume_jid: Some(job_id),
                        create_boss_volume_state: CreateBossVolumeState::WaitForJobCompletion,
                        iteration,
                    },
                },
            }
        }
        libredfish::JobState::ScheduledWithErrors | libredfish::JobState::CompletedWithErrors => {
            return handle_boss_controller_job_error(
                boss_controller_id,
                iteration.unwrap_or_default(),
                false,
                StateHandlerError::GenericError(eyre::eyre!(
                    "CreateBossVolume: job {} failed for {} with state {job_state:#?}",
                    job_id,
                    mh_snapshot.host_snapshot.id,
                )),
                mh_snapshot.host_snapshot.state.version.since_state_change(),
            );
        }
        _ => {
            return Ok(StateHandlerOutcome::wait(format!(
                "waiting for job {:#?} to be scheduled; current state: {job_state:#?}",
                job_id
            )));
        }
    };

    Ok(StateHandlerOutcome::transition(next_state))
}

async fn wait_for_boss_controller_job_to_complete(
    redfish_client: &dyn Redfish,
    mh_snapshot: &ManagedHostStateSnapshot,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let (boss_controller_id, boss_job_id, iterations, secure_erase_boss_controller) =
        match &mh_snapshot.host_snapshot.state.value {
            ManagedHostState::WaitingForCleanup { cleanup_state } => match cleanup_state {
                CleanupState::SecureEraseBoss {
                    secure_erase_boss_context,
                } => match &secure_erase_boss_context.secure_erase_boss_state {
                    SecureEraseBossState::WaitForJobCompletion => (
                        secure_erase_boss_context.boss_controller_id.clone(),
                        secure_erase_boss_context.secure_erase_jid.clone(),
                        secure_erase_boss_context.iteration.unwrap_or_default(),
                        // we are waiting for the secure erase job to complete
                        true,
                    ),
                    _ => {
                        return Err(StateHandlerError::GenericError(eyre::eyre!(
                            "unexpected SecureEraseBoss state for {}: {:#?}",
                            mh_snapshot.host_snapshot.id,
                            mh_snapshot.host_snapshot.state,
                        )));
                    }
                },
                CleanupState::CreateBossVolume {
                    create_boss_volume_context,
                } => match &create_boss_volume_context.create_boss_volume_state {
                    CreateBossVolumeState::WaitForJobCompletion => (
                        create_boss_volume_context.boss_controller_id.clone(),
                        create_boss_volume_context.create_boss_volume_jid.clone(),
                        create_boss_volume_context.iteration.unwrap_or_default(),
                        // we are waiting for the BOSS volume creation job to complete
                        false,
                    ),
                    _ => todo!(),
                },
                _ => {
                    return Err(StateHandlerError::GenericError(eyre::eyre!(
                        "unexpected CreateBossVolume state for {}: {:#?}",
                        mh_snapshot.host_snapshot.id,
                        mh_snapshot.host_snapshot.state,
                    )));
                }
            },
            _ => {
                return Err(StateHandlerError::GenericError(eyre::eyre!(
                    "unexpected host state for {}: {:#?}",
                    mh_snapshot.host_snapshot.id,
                    mh_snapshot.host_snapshot.state,
                )));
            }
        };

    let job_id = match boss_job_id {
        Some(jid) => Ok(jid),
        None => Err(StateHandlerError::GenericError(eyre::eyre!(
            "could not find job ID in the state's context"
        ))),
    }?;

    let job_state = match redfish_client.get_job_state(&job_id).await {
        Ok(state) => state,
        Err(e) => {
            return handle_boss_controller_job_error(
                boss_controller_id,
                iterations,
                secure_erase_boss_controller,
                StateHandlerError::RedfishError {
                    operation: "get_job_state",
                    error: e,
                },
                mh_snapshot.host_snapshot.state.version.since_state_change(),
            );
        }
    };

    match job_state {
        // The job has completed; transition to next step in host cleanup
        libredfish::JobState::Completed => {
            // healthy path
            let cleanup_state = match secure_erase_boss_controller {
                // now that we have finished doing a secure erase of the BOSS controller
                // we can do a standard secure erase of the remaining drives through the /usr/sbin/nvme tool
                true => CleanupState::HostCleanup {
                    boss_controller_id: Some(boss_controller_id),
                },
                // now that we have recreated the R1 volume on top of the BOSS controller, we can lock the host back down again.
                false => CleanupState::CreateBossVolume {
                    create_boss_volume_context: CreateBossVolumeContext {
                        boss_controller_id,
                        create_boss_volume_jid: None,
                        create_boss_volume_state: CreateBossVolumeState::LockHost,
                        iteration: Some(iterations),
                    },
                },
            };

            let next_state = ManagedHostState::WaitingForCleanup { cleanup_state };
            Ok(StateHandlerOutcome::transition(next_state))
        }
        // The job has failed; handle error
        libredfish::JobState::ScheduledWithErrors | libredfish::JobState::CompletedWithErrors => {
            handle_boss_controller_job_error(
                boss_controller_id,
                iterations,
                secure_erase_boss_controller,
                StateHandlerError::GenericError(eyre::eyre!(
                    "job {job_id} will not complete because it is in a failure state: {job_state:#?}",
                )),
                mh_snapshot.host_snapshot.state.version.since_state_change(),
            )
        }
        // The job is still running (hopefully...); wait for the job to complete
        _ => Ok(StateHandlerOutcome::wait(format!(
            "waiting for job {job_id} to complete; current state: {job_state:#?}"
        ))),
    }
}

async fn handle_boss_job_failure(
    redfish_client: &dyn Redfish,
    mh_snapshot: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let (next_state, expected_power_state) = get_next_state_boss_job_failure(mh_snapshot)?;

    let current_power_state =
        redfish_client
            .get_power_state()
            .await
            .map_err(|e| StateHandlerError::RedfishError {
                operation: "get_power_state",
                error: e,
            })?;

    match expected_power_state {
        libredfish::PowerState::Off => {
            if current_power_state != libredfish::PowerState::Off {
                handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceOff).await?;

                return Ok(StateHandlerOutcome::wait(format!(
                    "waiting for {} to power down; current power state: {current_power_state}",
                    mh_snapshot.host_snapshot.id
                )));
            }

            redfish_client
                .bmc_reset()
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "bmc_reset",
                    error: e,
                })?;

            Ok(StateHandlerOutcome::transition(next_state))
        }
        libredfish::PowerState::On => {
            let basetime = mh_snapshot
                .host_snapshot
                .last_reboot_requested
                .as_ref()
                .map(|x| x.time)
                .unwrap_or(mh_snapshot.host_snapshot.state.version.timestamp());

            if wait(
                &basetime,
                ctx.services
                    .site_config
                    .machine_state_controller
                    .power_down_wait,
            ) {
                return Ok(StateHandlerOutcome::wait(format!(
                    "waiting for {} to power down; power_down_wait: {}",
                    mh_snapshot.host_snapshot.id,
                    ctx.services
                        .site_config
                        .machine_state_controller
                        .power_down_wait
                )));
            }

            if current_power_state != libredfish::PowerState::On {
                handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::On).await?;

                return Ok(StateHandlerOutcome::wait(format!(
                    "waiting for {} to power on; current power state: {current_power_state}",
                    mh_snapshot.host_snapshot.id,
                )));
            }

            Ok(StateHandlerOutcome::transition(next_state))
        }
        _ => Err(StateHandlerError::GenericError(eyre::eyre!(
            "unexpected expected_power_state while handling a boss job failure: {expected_power_state}"
        ))),
    }
}
