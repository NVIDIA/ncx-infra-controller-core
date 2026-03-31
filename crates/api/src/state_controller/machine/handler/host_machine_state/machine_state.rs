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
use libredfish::SystemPowerControl;
use model::machine::{
    BomValidating, BomValidatingContext, LockdownInfo, LockdownMode, LockdownState, MachineState,
    ManagedHostState, ManagedHostStateSnapshot, SetBootOrderInfo, SetBootOrderState, UefiSetupInfo,
    UefiSetupState,
};

use super::super::managed_host_state::measuring::map_host_init_measuring_outcome_to_state_handler_outcome;
use super::super::{
    BiosConfigOutcome, HostHandlerParams, are_dpus_up_trigger_reboot_if_needed,
    configure_host_bios, discovered_after_state_transition, handler_host_power_control, rebooted,
    trigger_reboot_if_needed, wait,
};
use super::handle_host_boot_order_setup::handle_host_boot_order_setup;
use super::handle_host_uefi_setup::handle_host_uefi_setup;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::handle_measuring_state;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle(
    host_machine_id: &MachineId,
    mh_snapshot: &mut ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    host_handler_params: &HostHandlerParams,
    machine_state: &MachineState,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    match machine_state {
        MachineState::Init => Err(StateHandlerError::InvalidHostState(
            *host_machine_id,
            Box::new(mh_snapshot.managed_state.clone()),
        )),
        MachineState::EnableIpmiOverLan => {
            let host_redfish_client = ctx
                .services
                .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
                .await?;

            if !host_redfish_client
                .is_ipmi_over_lan_enabled()
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "enable_ipmi_over_lan",
                    error: e,
                })?
            {
                tracing::info!(
                    machine_id = %host_machine_id,
                    "IPMI over LAN is currently disabled on this host--enabling IPMI over LAN");

                host_redfish_client
                    .enable_ipmi_over_lan(libredfish::EnabledDisabled::Enabled)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "enable_ipmi_over_lan",
                        error: e,
                    })?;
            }

            let next_state = ManagedHostState::HostInit {
                machine_state: MachineState::WaitingForPlatformConfiguration,
            };

            Ok(StateHandlerOutcome::transition(next_state))
        }
        MachineState::WaitingForPlatformConfiguration => {
            tracing::info!(
                machine_id = %host_machine_id,
                "Starting UEFI / BMC setup");

            let redfish_client = ctx
                .services
                .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
                .await?;

            match redfish_client.lockdown_status().await {
                Err(libredfish::RedfishError::NotSupported(_)) => {
                    tracing::info!(
                        "BMC vendor does not support checking lockdown status for {host_machine_id}."
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Error fetching lockdown status for {host_machine_id} during machine_setup check: {e}"
                    );
                    return Ok(StateHandlerOutcome::wait(format!(
                        "Failed to fetch lockdown status: {}",
                        e
                    )));
                }
                Ok(lockdown_status) if !lockdown_status.is_fully_disabled() => {
                    tracing::info!(
                        "Lockdown is enabled for {host_machine_id} during machine_setup, disabling now."
                    );
                    let next_state = ManagedHostState::HostInit {
                        machine_state: MachineState::WaitingForLockdown {
                            lockdown_info: LockdownInfo {
                                state: LockdownState::SetLockdown,
                                mode: LockdownMode::Disable,
                            },
                        },
                    };
                    return Ok(StateHandlerOutcome::transition(next_state));
                }
                Ok(_) => {
                    // Lockdown is disabled, proceed with machine_setup
                }
            }

            match configure_host_bios(
                ctx,
                &host_handler_params.reachability_params,
                redfish_client.as_ref(),
                mh_snapshot,
            )
            .await?
            {
                BiosConfigOutcome::Done => {
                    // BIOS configuration done, move to polling
                    Ok(StateHandlerOutcome::transition(
                        ManagedHostState::HostInit {
                            machine_state: MachineState::PollingBiosSetup,
                        },
                    ))
                }
                BiosConfigOutcome::WaitingForReboot(reason) => {
                    Ok(StateHandlerOutcome::wait(reason))
                }
            }
        }
        MachineState::PollingBiosSetup => {
            let next_state = ManagedHostState::HostInit {
                machine_state: MachineState::SetBootOrder {
                    set_boot_order_info: Some(SetBootOrderInfo {
                        set_boot_order_jid: None,
                        set_boot_order_state: SetBootOrderState::SetBootOrder,
                        retry_count: 0,
                    }),
                },
            };

            let redfish_client = ctx
                .services
                .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
                .await?;

            let boot_interface_mac = if !mh_snapshot.dpu_snapshots.is_empty() {
                let primary_interface = mh_snapshot
                    .host_snapshot
                    .interfaces
                    .iter()
                    .find(|x| x.primary_interface)
                    .ok_or_else(|| {
                        StateHandlerError::GenericError(eyre::eyre!(
                            "Missing primary interface from host: {}",
                            mh_snapshot.host_snapshot.id
                        ))
                    })?;
                Some(primary_interface.mac_address.to_string())
            } else {
                None
            };

            match redfish_client
                .is_bios_setup(boot_interface_mac.as_deref())
                .await
            {
                Ok(true) => {
                    tracing::info!(
                        machine_id = %mh_snapshot.host_snapshot.id,
                        "BIOS setup verified successfully"
                    );
                    Ok(StateHandlerOutcome::transition(next_state))
                }
                Ok(false) => Ok(StateHandlerOutcome::wait(
                    "Polling BIOS setup status, waiting for settings to be applied".to_string(),
                )),
                Err(e) => {
                    tracing::warn!(
                        machine_id = %mh_snapshot.host_snapshot.id,
                        error = %e,
                        "Failed to check BIOS setup status, will retry"
                    );
                    Ok(StateHandlerOutcome::wait(format!(
                        "Failed to check BIOS setup status: {}. Will retry.",
                        e
                    )))
                }
            }
        }
        MachineState::SetBootOrder {
            set_boot_order_info,
        } => Ok(handle_host_boot_order_setup(
            ctx,
            host_handler_params.clone(),
            mh_snapshot,
            set_boot_order_info.clone(),
        )
        .await?),
        MachineState::Measuring { measuring_state } => {
            match handle_measuring_state(
                measuring_state,
                &mh_snapshot.host_snapshot.id,
                &mut ctx.services.db_reader,
                host_handler_params.attestation_enabled,
            )
            .await
            {
                Ok(measuring_outcome) => map_host_init_measuring_outcome_to_state_handler_outcome(
                    &measuring_outcome,
                    measuring_state,
                ),
                Err(StateHandlerError::MissingData {
                    object_id: _,
                    missing: "ek_cert_verification_status",
                }) => Ok(StateHandlerOutcome::wait(
                    "Waiting for Scout to start and send registration info (in discover_machine)"
                        .to_string(),
                )),
                Err(e) => Err(e),
            }
        }
        MachineState::WaitingForDiscovery => {
            if !discovered_after_state_transition(
                mh_snapshot.host_snapshot.state.version,
                mh_snapshot.host_snapshot.last_discovery_time,
            ) {
                tracing::trace!(
                    machine_id = %host_machine_id,
                    "Waiting for forge-scout to report host online. \
                                 Host last seen {:?}, must come after DPU's {}",
                    mh_snapshot.host_snapshot.last_discovery_time,
                    mh_snapshot.host_snapshot.state.version.timestamp()
                );
                let status = trigger_reboot_if_needed(
                    &mh_snapshot.host_snapshot,
                    mh_snapshot,
                    None,
                    &host_handler_params.reachability_params,
                    ctx,
                )
                .await?;
                return Ok(StateHandlerOutcome::wait(status.status));
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
        MachineState::UefiSetup { uefi_setup_info } => {
            Ok(handle_host_uefi_setup(ctx, mh_snapshot, uefi_setup_info.clone()).await?)
        }
        MachineState::WaitingForLockdown { lockdown_info } => {
            match &lockdown_info.state {
                LockdownState::SetLockdown => {
                    tracing::info!(
                        machine_id = %host_machine_id,
                        mode = ?lockdown_info.mode,
                        "Setting lockdown and issuing reboot"
                    );

                    let redfish_client = ctx
                        .services
                        .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
                        .await?;

                    let action = match lockdown_info.mode {
                        LockdownMode::Enable => libredfish::EnabledDisabled::Enabled,
                        LockdownMode::Disable => libredfish::EnabledDisabled::Disabled,
                    };

                    redfish_client.lockdown(action).await.map_err(|e| {
                        StateHandlerError::RedfishError {
                            operation: "lockdown",
                            error: e,
                        }
                    })?;

                    handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart)
                        .await?;

                    Ok(StateHandlerOutcome::transition(
                        ManagedHostState::HostInit {
                            machine_state: MachineState::WaitingForLockdown {
                                lockdown_info: LockdownInfo {
                                    state: LockdownState::TimeWaitForDPUDown,
                                    mode: lockdown_info.mode.clone(),
                                },
                            },
                        },
                    ))
                }
                LockdownState::TimeWaitForDPUDown => {
                    if ctx.services.site_config.force_dpu_nic_mode {
                        // skip wait for dpu reboot TimeWaitForDPUDown, WaitForDPUUp
                        // GB200/300, etc with dpu disconnected or in nic mode
                        let next_state = ManagedHostState::BomValidating {
                            bom_validating_state: BomValidating::MatchingSku(
                                BomValidatingContext {
                                    machine_validation_context: Some("Discovery".to_string()),
                                    reboot_retry_count: None,
                                },
                            ),
                        };
                        return Ok(StateHandlerOutcome::transition(next_state));
                    }
                    // Lets wait for some time before checking if DPU is up or not.
                    // Waiting is needed because DPU takes some time to go down. If we check DPU
                    // reachability before it goes down, it will give us wrong result.
                    if wait(
                        &mh_snapshot.host_snapshot.state.version.timestamp(),
                        host_handler_params.reachability_params.dpu_wait_time,
                    ) {
                        Ok(StateHandlerOutcome::wait(format!(
                            "Forced wait of {} for DPU to power down",
                            host_handler_params.reachability_params.dpu_wait_time
                        )))
                    } else {
                        let next_state = ManagedHostState::HostInit {
                            machine_state: MachineState::WaitingForLockdown {
                                lockdown_info: LockdownInfo {
                                    state: LockdownState::WaitForDPUUp,
                                    mode: lockdown_info.mode.clone(),
                                },
                            },
                        };
                        Ok(StateHandlerOutcome::transition(next_state))
                    }
                }
                LockdownState::WaitForDPUUp => {
                    // Has forge-dpu-agent reported state? That means DPU is up.
                    if are_dpus_up_trigger_reboot_if_needed(
                        mh_snapshot,
                        &host_handler_params.reachability_params,
                        ctx,
                    )
                    .await
                    {
                        // reboot host
                        // When forge changes BIOS params (for lockdown enable/disable both), host does a power cycle.
                        // During power cycle, DPU also reboots. Now DPU and Host are coming up together. Since DPU is not ready yet,
                        // it does not forward DHCP discover from host and host goes into failure mode and stops sending further
                        // DHCP Discover. A second reboot starts DHCP cycle again when DPU is already up.

                        handler_host_power_control(
                            mh_snapshot,
                            ctx,
                            SystemPowerControl::ForceRestart,
                        )
                        .await?;

                        let next_state = ManagedHostState::HostInit {
                            machine_state: MachineState::WaitingForLockdown {
                                lockdown_info: LockdownInfo {
                                    state: LockdownState::PollingLockdownStatus,
                                    mode: lockdown_info.mode.clone(),
                                },
                            },
                        };
                        Ok(StateHandlerOutcome::transition(next_state))
                    } else {
                        Ok(StateHandlerOutcome::wait("Waiting for DPU to report UP. This requires forge-dpu-agent to call the RecordDpuNetworkStatus API".to_string()))
                    }
                }
                LockdownState::PollingLockdownStatus => {
                    let next_state = if LockdownMode::Enable == lockdown_info.mode {
                        ManagedHostState::BomValidating {
                            bom_validating_state: BomValidating::MatchingSku(
                                BomValidatingContext {
                                    machine_validation_context: Some("Discovery".to_string()),
                                    ..BomValidatingContext::default()
                                },
                            ),
                        }
                    } else {
                        ManagedHostState::HostInit {
                            machine_state: MachineState::WaitingForPlatformConfiguration,
                        }
                    };

                    let redfish_client = ctx
                        .services
                        .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
                        .await?;

                    match redfish_client.lockdown_status().await {
                        Ok(lockdown_status) => {
                            let expected_state = match lockdown_info.mode {
                                LockdownMode::Enable => lockdown_status.is_fully_enabled(),
                                LockdownMode::Disable => lockdown_status.is_fully_disabled(),
                            };

                            if expected_state {
                                tracing::info!(
                                    machine_id = %mh_snapshot.host_snapshot.id,
                                    mode = ?lockdown_info.mode,
                                    "Lockdown status verified successfully"
                                );
                                Ok(StateHandlerOutcome::transition(next_state))
                            } else {
                                Ok(StateHandlerOutcome::wait(format!(
                                    "Polling lockdown status, waiting for {:?} to be applied. Current status: {:?}",
                                    lockdown_info.mode, lockdown_status
                                )))
                            }
                        }
                        Err(libredfish::RedfishError::NotSupported(_)) => {
                            tracing::info!(
                                "BMC vendor does not support checking lockdown status for {host_machine_id}."
                            );
                            Ok(StateHandlerOutcome::transition(next_state))
                        }
                        Err(e) => {
                            tracing::warn!(
                                machine_id = %mh_snapshot.host_snapshot.id,
                                error = %e,
                                "Failed to check lockdown status, will retry"
                            );
                            Ok(StateHandlerOutcome::wait(format!(
                                "Failed to check lockdown status: {}. Will retry.",
                                e
                            )))
                        }
                    }
                }
            }
        }
        MachineState::Discovered {
            skip_reboot_wait: skip_reboot,
        } => {
            // Check if machine is rebooted. If yes, move to Ready state
            // or Measuring state, depending on if machine attestation
            // is enabled or not.
            if rebooted(&mh_snapshot.host_snapshot) || *skip_reboot {
                Ok(StateHandlerOutcome::transition(ManagedHostState::Ready))
            } else {
                let status = trigger_reboot_if_needed(
                    &mh_snapshot.host_snapshot,
                    mh_snapshot,
                    None,
                    &host_handler_params.reachability_params,
                    ctx,
                )
                .await?;
                Ok(StateHandlerOutcome::wait(format!(
                    "Waiting for scout to call RebootCompleted grpc. {}",
                    status.status
                )))
            }
        }
    }
}
