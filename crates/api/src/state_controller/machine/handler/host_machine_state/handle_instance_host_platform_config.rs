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
use libredfish::{EnabledDisabled, PowerState, SystemPowerControl};
use model::machine::{
    HostPlatformConfigurationState, InstanceState, ManagedHostState, ManagedHostStateSnapshot,
    SetBootOrderInfo, SetBootOrderState, UnlockHostState,
};

use super::super::{
    BiosConfigOutcome, ReachabilityParams, SetBootOrderOutcome,
    are_dpus_up_trigger_reboot_if_needed, configure_host_bios, log_host_config,
    set_host_boot_order, wait,
};
use crate::CarbideError;
use crate::redfish::host_power_control;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(crate) async fn handle_instance_host_platform_config(
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    mh_snapshot: &mut ManagedHostStateSnapshot,
    reachability_params: &ReachabilityParams,
    platform_config_state: HostPlatformConfigurationState,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let redfish_client = ctx
        .services
        .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
        .await?;

    let instance_state = match platform_config_state {
        HostPlatformConfigurationState::PowerCycle {
            power_on,
            power_on_retry_count,
        } => {
            let power_state = redfish_client.get_power_state().await.map_err(|e| {
                StateHandlerError::RedfishError {
                    operation: "get_power_state",
                    error: e,
                }
            })?;

            // Phase 1: Power OFF (power_on=false means we need to power off first)
            if !power_on {
                if power_state == PowerState::Off {
                    // Host is already off, proceed to power on phase
                    return Ok(StateHandlerOutcome::transition(
                        ManagedHostState::Assigned {
                            instance_state: InstanceState::HostPlatformConfiguration {
                                platform_config_state: HostPlatformConfigurationState::PowerCycle {
                                    power_on: true,
                                    power_on_retry_count: 0,
                                },
                            },
                        },
                    ));
                }

                // Host is still on, issue power off command
                host_power_control(
                    redfish_client.as_ref(),
                    &mh_snapshot.host_snapshot,
                    SystemPowerControl::ForceOff,
                    ctx,
                )
                .await
                .map_err(|e| {
                    StateHandlerError::GenericError(eyre!("failed to power off host: {}", e))
                })?;

                return Ok(StateHandlerOutcome::wait(format!(
                    "waiting for {} to power OFF; current power state: {}",
                    mh_snapshot.host_snapshot.id, power_state
                )));
            }

            // Phase 2: Power ON (power_on=true means host was off, now power it on)

            // Wait for the power-down grace period before powering back on
            let basetime = mh_snapshot
                .host_snapshot
                .last_reboot_requested
                .as_ref()
                .map(|x| x.time)
                .unwrap_or(mh_snapshot.host_snapshot.state.version.timestamp());

            if wait(&basetime, reachability_params.power_down_wait) {
                return Ok(StateHandlerOutcome::wait(format!(
                    "waiting for power-down grace period before powering on {}; power_down_wait: {}",
                    mh_snapshot.host_snapshot.id, reachability_params.power_down_wait
                )));
            }

            if power_state == PowerState::On {
                // Host is on, unlock BMC before checking config so Redfish reflects reality
                return Ok(StateHandlerOutcome::transition(
                    ManagedHostState::Assigned {
                        instance_state: InstanceState::HostPlatformConfiguration {
                            platform_config_state: HostPlatformConfigurationState::UnlockHost {
                                unlock_host_state: UnlockHostState::DisableLockdown,
                            },
                        },
                    },
                ));
            }

            // Host is still off. Every 5th retry use AC power cycle instead of On.
            let next_retry = power_on_retry_count + 1;
            if next_retry % 5 == 0 {
                match host_power_control(
                    redfish_client.as_ref(),
                    &mh_snapshot.host_snapshot,
                    SystemPowerControl::ACPowercycle,
                    ctx,
                )
                .await
                {
                    Ok(()) => {
                        return Ok(StateHandlerOutcome::transition(
                            ManagedHostState::Assigned {
                                instance_state: InstanceState::HostPlatformConfiguration {
                                    platform_config_state:
                                        HostPlatformConfigurationState::PowerCycle {
                                            power_on: true,
                                            power_on_retry_count: next_retry,
                                        },
                                },
                            },
                        ));
                    }
                    Err(CarbideError::RedfishError(libredfish::RedfishError::NotSupported(_))) => {
                        // if not supported, just power on
                        tracing::info!("AC Powercycle not supported, skipping to power on");
                    }
                    Err(e) => {
                        // TODO: Dell's return a generic error if in lockdown which needs to be changed in Redfish SDK
                        tracing::warn!("Failed to AC Powercycle host, skipping to power on: {e}");
                    }
                };
            }

            host_power_control(
                redfish_client.as_ref(),
                &mh_snapshot.host_snapshot,
                SystemPowerControl::On,
                ctx,
            )
            .await
            .map_err(|e| StateHandlerError::GenericError(eyre!("failed to power on host: {e}")))?;

            tracing::info!(
                host_id = %mh_snapshot.host_snapshot.id,
                power_on_retry_count = next_retry,
                %power_state,
                "waiting for host to power ON"
            );
            return Ok(StateHandlerOutcome::transition(
                ManagedHostState::Assigned {
                    instance_state: InstanceState::HostPlatformConfiguration {
                        platform_config_state: HostPlatformConfigurationState::PowerCycle {
                            power_on: true,
                            power_on_retry_count: next_retry,
                        },
                    },
                },
            ));
        }
        HostPlatformConfigurationState::UnlockHost { unlock_host_state } => {
            match unlock_host_state {
                UnlockHostState::DisableLockdown => {
                    redfish_client
                        .lockdown_bmc(EnabledDisabled::Disabled)
                        .await
                        .map_err(|e| StateHandlerError::RedfishError {
                            operation: "lockdown_bmc",
                            error: e,
                        })?;

                    let vendor = mh_snapshot.host_snapshot.bmc_vendor();

                    // Supermicro BMCs in lockdown mode sometimes report stale boot order
                    // via Redfish (https://github.com/NVIDIA/bare-metal-manager-core/issues/505).
                    // A reboot with lockdown disabled forces the BMC to re-read the actual UEFI
                    // boot configuration.
                    if vendor.is_supermicro() {
                        tracing::info!(
                            machine_id = %mh_snapshot.host_snapshot.id,
                            %vendor,
                            "BMC lockdown disabled; rebooting host so Redfish reflects actual boot order"
                        );
                        InstanceState::HostPlatformConfiguration {
                            platform_config_state: HostPlatformConfigurationState::UnlockHost {
                                unlock_host_state: UnlockHostState::RebootHost,
                            },
                        }
                    } else {
                        tracing::info!(
                            machine_id = %mh_snapshot.host_snapshot.id,
                            %vendor,
                            "BMC lockdown disabled; skipping post-unlock reboot (not required for this vendor)"
                        );
                        InstanceState::HostPlatformConfiguration {
                            platform_config_state: HostPlatformConfigurationState::CheckHostConfig,
                        }
                    }
                }
                UnlockHostState::RebootHost => {
                    host_power_control(
                        redfish_client.as_ref(),
                        &mh_snapshot.host_snapshot,
                        SystemPowerControl::ForceRestart,
                        ctx,
                    )
                    .await
                    .map_err(|e| {
                        StateHandlerError::GenericError(eyre!(
                            "failed to ForceRestart host after disabling BMC lockdown: {}",
                            e
                        ))
                    })?;

                    InstanceState::HostPlatformConfiguration {
                        platform_config_state: HostPlatformConfigurationState::UnlockHost {
                            unlock_host_state: UnlockHostState::WaitForUefiBoot,
                        },
                    }
                }
                UnlockHostState::WaitForUefiBoot => {
                    let entered_at = mh_snapshot.host_snapshot.state.version.timestamp();
                    if wait(&entered_at, reachability_params.uefi_boot_wait) {
                        return Ok(StateHandlerOutcome::wait(format!(
                            "Waiting for UEFI boot to complete on {} after post-unlock reboot; \
                             wait duration: {}, will proceed after {}",
                            mh_snapshot.host_snapshot.id,
                            reachability_params.uefi_boot_wait,
                            entered_at + reachability_params.uefi_boot_wait,
                        )));
                    }

                    InstanceState::HostPlatformConfiguration {
                        platform_config_state: HostPlatformConfigurationState::CheckHostConfig,
                    }
                }
            }
        }
        HostPlatformConfigurationState::CheckHostConfig => {
            let configure_host_boot_order = if !mh_snapshot.dpu_snapshots.is_empty() {
                // Given that we are checking the boot order of a server immediately after a power cycle, we
                // should do some waiting to ensure that the host is not reporting stale redfish information from
                // before Carbide powered it off.
                // This check guarantees that the host has finished loading the BIOS after the DPUs have come up.
                // If Carbide is still reading an incorrect boot order at this point, something is wrong, and
                // we should configure this host properly.
                if !are_dpus_up_trigger_reboot_if_needed(mh_snapshot, reachability_params, ctx)
                    .await
                {
                    return Ok(StateHandlerOutcome::wait(
                        "Waiting for DPUs to come up.".to_string(),
                    ));
                }

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

                let vendor = mh_snapshot.host_snapshot.bmc_vendor();

                log_host_config(redfish_client.as_ref(), mh_snapshot).await;

                if !(redfish_client
                    .is_boot_order_setup(&primary_interface.mac_address.to_string())
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "is_boot_order_setup",
                        error: e,
                    })?)
                {
                    tracing::warn!(
                        machine_id = %mh_snapshot.host_snapshot.id,
                        bmc_vendor = %vendor,
                        "Host boot order is not configured properly"
                    );

                    true
                } else {
                    tracing::info!(
                        machine_id = %mh_snapshot.host_snapshot.id,
                        bmc_vendor = %vendor,
                        "Host boot order is configured properly"
                    );

                    false
                }
            } else {
                false
            };

            if configure_host_boot_order {
                InstanceState::HostPlatformConfiguration {
                    platform_config_state: HostPlatformConfigurationState::ConfigureBios,
                }
            } else {
                // Boot order is already correct (or no DPUs); skip to LockHost to
                // re-enable BMC lockdown before proceeding.
                InstanceState::HostPlatformConfiguration {
                    platform_config_state: HostPlatformConfigurationState::LockHost,
                }
            }
        }
        HostPlatformConfigurationState::ConfigureBios => {
            match configure_host_bios(
                ctx,
                reachability_params,
                redfish_client.as_ref(),
                mh_snapshot,
            )
            .await?
            {
                BiosConfigOutcome::Done => {
                    // BIOS configuration done, move to polling
                    return Ok(StateHandlerOutcome::transition(
                        ManagedHostState::Assigned {
                            instance_state: InstanceState::HostPlatformConfiguration {
                                platform_config_state:
                                    HostPlatformConfigurationState::PollingBiosSetup,
                            },
                        },
                    ));
                }
                BiosConfigOutcome::WaitingForReboot(reason) => {
                    return Ok(StateHandlerOutcome::wait(reason));
                }
            }
        }
        HostPlatformConfigurationState::PollingBiosSetup => {
            let next_instance_state = InstanceState::HostPlatformConfiguration {
                platform_config_state: HostPlatformConfigurationState::SetBootOrder {
                    set_boot_order_info: SetBootOrderInfo {
                        set_boot_order_jid: None,
                        set_boot_order_state: SetBootOrderState::SetBootOrder,
                        retry_count: 0,
                    },
                },
            };

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
                    next_instance_state
                }
                Ok(false) => {
                    return Ok(StateHandlerOutcome::wait(
                        "Polling BIOS setup status, waiting for settings to be applied".to_string(),
                    ));
                }
                Err(e) => {
                    tracing::warn!(
                        machine_id = %mh_snapshot.host_snapshot.id,
                        error = %e,
                        "Failed to check BIOS setup status, will retry"
                    );
                    return Ok(StateHandlerOutcome::wait(format!(
                        "Failed to check BIOS setup status: {}. Will retry.",
                        e
                    )));
                }
            }
        }
        HostPlatformConfigurationState::SetBootOrder {
            set_boot_order_info,
        } => {
            match set_host_boot_order(
                ctx,
                reachability_params,
                redfish_client.as_ref(),
                mh_snapshot,
                set_boot_order_info,
            )
            .await?
            {
                SetBootOrderOutcome::Continue(boot_order_info) => {
                    InstanceState::HostPlatformConfiguration {
                        platform_config_state: HostPlatformConfigurationState::SetBootOrder {
                            set_boot_order_info: boot_order_info,
                        },
                    }
                }
                SetBootOrderOutcome::Done => InstanceState::HostPlatformConfiguration {
                    platform_config_state: HostPlatformConfigurationState::LockHost,
                },
                SetBootOrderOutcome::WaitingForReboot(reason) => {
                    return Ok(StateHandlerOutcome::wait(reason));
                }
            }
        }
        HostPlatformConfigurationState::LockHost => {
            redfish_client
                .lockdown_bmc(EnabledDisabled::Enabled)
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "lockdown_bmc",
                    error: e,
                })?;

            InstanceState::WaitingForDpusToUp
        }
    };

    let next_state = ManagedHostState::Assigned { instance_state };

    Ok(StateHandlerOutcome::transition(next_state))
}
