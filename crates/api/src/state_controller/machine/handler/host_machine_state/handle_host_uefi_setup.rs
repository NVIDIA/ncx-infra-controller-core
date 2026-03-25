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

use eyre::eyre;
use libredfish::SystemPowerControl;
use model::machine::{
    LockdownInfo, LockdownMode, LockdownState, MachineState, ManagedHostState,
    ManagedHostStateSnapshot, UefiSetupInfo, UefiSetupState,
};

use super::super::handler_host_power_control;
use crate::redfish::set_host_uefi_password;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

/// TODO: we need to handle the case where the job is deleted for some reason
pub(crate) async fn handle_host_uefi_setup(
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    state: &mut ManagedHostStateSnapshot,
    uefi_setup_info: UefiSetupInfo,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let redfish_client = ctx
        .services
        .create_redfish_client_from_machine(&state.host_snapshot)
        .await?;

    match uefi_setup_info.uefi_setup_state.clone() {
        UefiSetupState::UnlockHost => {
            if state.host_snapshot.bmc_vendor().is_dell() {
                redfish_client
                    .lockdown_bmc(libredfish::EnabledDisabled::Disabled)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "lockdown",
                        error: e,
                    })?;
            }

            Ok(StateHandlerOutcome::transition(
                ManagedHostState::HostInit {
                    machine_state: MachineState::UefiSetup {
                        uefi_setup_info: UefiSetupInfo {
                            uefi_password_jid: None,
                            uefi_setup_state: UefiSetupState::SetUefiPassword,
                        },
                    },
                },
            ))
        }
        UefiSetupState::SetUefiPassword => {
            match set_host_uefi_password(
                redfish_client.as_ref(),
                ctx.services.redfish_client_pool.clone(),
            )
            .await
            {
                Ok(job_id) => Ok(StateHandlerOutcome::transition(
                    ManagedHostState::HostInit {
                        machine_state: MachineState::UefiSetup {
                            uefi_setup_info: UefiSetupInfo {
                                uefi_password_jid: job_id,
                                uefi_setup_state: UefiSetupState::WaitForPasswordJobScheduled,
                            },
                        },
                    },
                )),
                Err(e) => {
                    let msg = format!(
                        "failed to set the BIOS password on {} ({}): {}",
                        state.host_snapshot.id,
                        state.host_snapshot.bmc_vendor(),
                        e
                    );

                    // This feature has only been tested thoroughly on Dells, Lenovos, and Vikings.
                    if state.host_snapshot.bmc_vendor().is_dell()
                        || state.host_snapshot.bmc_vendor().is_lenovo()
                        || state.host_snapshot.bmc_vendor().is_nvidia()
                    {
                        return Err(StateHandlerError::GenericError(eyre::eyre!("{}", msg)));
                    }

                    // For all other vendors, allow ingestion even though we couldnt set the bios password
                    // An operator will have to set the bios password manually
                    tracing::info!(msg);

                    Ok(StateHandlerOutcome::transition(
                        ManagedHostState::HostInit {
                            machine_state: MachineState::WaitingForLockdown {
                                lockdown_info: LockdownInfo {
                                    state: LockdownState::SetLockdown,
                                    mode: LockdownMode::Enable,
                                },
                            },
                        },
                    ))
                }
            }
        }
        UefiSetupState::WaitForPasswordJobScheduled => {
            if let Some(job_id) = uefi_setup_info.uefi_password_jid.clone() {
                let job_state = redfish_client.get_job_state(&job_id).await.map_err(|e| {
                    StateHandlerError::RedfishError {
                        operation: "get_job_state",
                        error: e,
                    }
                })?;

                if !matches!(job_state, libredfish::JobState::Scheduled) {
                    return Ok(StateHandlerOutcome::wait(format!(
                        "waiting for job {:#?} to be scheduled; current state: {job_state:#?}",
                        job_id
                    )));
                }
            }

            Ok(StateHandlerOutcome::transition(
                ManagedHostState::HostInit {
                    machine_state: MachineState::UefiSetup {
                        uefi_setup_info: UefiSetupInfo {
                            uefi_password_jid: uefi_setup_info.uefi_password_jid.clone(),
                            uefi_setup_state: UefiSetupState::PowercycleHost,
                        },
                    },
                },
            ))
        }
        UefiSetupState::PowercycleHost => {
            handler_host_power_control(state, ctx, SystemPowerControl::ForceRestart).await?;
            Ok(StateHandlerOutcome::transition(
                ManagedHostState::HostInit {
                    machine_state: MachineState::UefiSetup {
                        uefi_setup_info: UefiSetupInfo {
                            uefi_password_jid: uefi_setup_info.uefi_password_jid.clone(),
                            uefi_setup_state: UefiSetupState::WaitForPasswordJobCompletion,
                        },
                    },
                },
            ))
        }
        UefiSetupState::WaitForPasswordJobCompletion => {
            if let Some(job_id) = uefi_setup_info.uefi_password_jid.clone() {
                let redfish_client = ctx
                    .services
                    .create_redfish_client_from_machine(&state.host_snapshot)
                    .await?;

                let job_state = redfish_client.get_job_state(&job_id).await.map_err(|e| {
                    StateHandlerError::RedfishError {
                        operation: "get_job_state",
                        error: e,
                    }
                })?;

                if !matches!(job_state, libredfish::JobState::Completed) {
                    return Ok(StateHandlerOutcome::wait(format!(
                        "waiting for job {:#?} to complete; current state: {job_state:#?}",
                        job_id
                    )));
                }
            }

            let mut txn = ctx.services.db_pool.begin().await?;
            state.host_snapshot.bios_password_set_time = Some(chrono::offset::Utc::now());
            db::machine::update_bios_password_set_time(&state.host_snapshot.id, &mut txn)
                .await
                .map_err(|e| {
                    StateHandlerError::GenericError(eyre!(
                        "update_host_bios_password_set failed: {}",
                        e
                    ))
                })?;

            Ok(StateHandlerOutcome::transition(ManagedHostState::HostInit {
                machine_state: MachineState::WaitingForLockdown {
                    lockdown_info: LockdownInfo {
                        state: LockdownState::SetLockdown,
                        mode: LockdownMode::Enable,
                    },
                },
            })
            .with_txn(txn))
        }
        // Deprecated: Kept for backwards compatibility with hosts that may be in this state.
        UefiSetupState::LockdownHost => Ok(StateHandlerOutcome::transition(
            ManagedHostState::HostInit {
                machine_state: MachineState::WaitingForLockdown {
                    lockdown_info: LockdownInfo {
                        state: LockdownState::SetLockdown,
                        mode: LockdownMode::Enable,
                    },
                },
            },
        )),
    }
}
