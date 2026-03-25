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

use super::*;

impl HostUpgradeState {
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn host_checking_fw(
        &self,
        details: &HostReprovisionState,
        state: &ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
        original_state: &ManagedHostState,
        scenario: HostFirmwareScenario,
        repeat: bool,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let machine_id = state.host_snapshot.id;
        let ret = self
            .host_checking_fw_noclear(details, state, ctx, &machine_id, scenario, repeat)
            .await?;

        // Check if we are returning to the ready state, and clear the host reprovisioning request if so.
        let mut ret = match ret {
            StateHandlerOutcome::Transition {
                next_state:
                    ManagedHostState::HostReprovision { .. }
                    | ManagedHostState::Assigned {
                        instance_state: InstanceState::HostReprovision { .. },
                    },
                ..
            } => ret,
            _ => {
                ret.in_transaction(&ctx.services.db_pool, move |txn| {
                    async move {
                        db::host_machine_update::clear_host_reprovisioning_request(
                            txn,
                            &machine_id,
                        )
                        .await?;
                        // TODO: Remove when manual upgrade feature is removed
                        db::host_machine_update::clear_manual_firmware_upgrade_completed(
                            txn,
                            &machine_id,
                        )
                        .await?;
                        Ok::<_, DatabaseError>(())
                    }
                    .boxed()
                })
                .await??
            }
        };

        if let StateHandlerOutcome::Transition { next_state, .. } = &ret
            && next_state == original_state
        {
            // host_checking_fw_noclear can return Ready to indicate that we're moving out of CheckingFirmware,
            // but we also take this path when we're actually in Ready - for that case, return do_nothing() so that
            // we don't keep retransitioning to the same state.
            return Ok(StateHandlerOutcome::do_nothing().with_txn_opt(ret.take_transaction()));
        }

        Ok(ret)
    }

    #[allow(clippy::too_many_arguments)]
    async fn host_checking_fw_noclear(
        &self,
        details: &HostReprovisionState,
        state: &ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
        machine_id: &MachineId,
        scenario: HostFirmwareScenario,
        repeat: bool,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        // temporary check if manual upgrade is required before proceeding with automatic ones,
        // should be removed once we complete upgrades through the scout.
        // For now, only gb200s need manual upgrades.
        if requires_manual_firmware_upgrade(state, &ctx.services.site_config) {
            tracing::info!(
                "Machine {} (GB200) requires manual firmware upgrade, transitioning to WaitingForManualUpgrade",
                machine_id
            );
            return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                HostReprovisionState::WaitingForManualUpgrade {
                    manual_upgrade_started: Utc::now(),
                },
                state.managed_state.get_host_repro_retry_count(),
            )));
        }

        let (current_firmware_type, current_firmware_number): (Option<FirmwareComponentType>, u32) =
            match details {
                HostReprovisionState::CheckingFirmwareV2 {
                    firmware_number,
                    firmware_type,
                }
                | HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_number,
                    firmware_type,
                } => (*firmware_type, firmware_number.unwrap_or(0)),
                _ => {
                    return Err(StateHandlerError::GenericError(eyre!(
                        "Wrong enum in host_checking_fw_noclear"
                    )));
                }
            };

        let Some(explored_endpoint) =
            find_explored_refreshed_endpoint(state, machine_id, ctx).await?
        else {
            // find_explored_refreshed_endpoint's behavior is to return None to indicate we're waiting for an update, not to indicate there isn't anything.

            tracing::debug!("Managed host {machine_id} waiting for site explorer to revisit");
            return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_type: current_firmware_type,
                    firmware_number: Some(current_firmware_number),
                },
                state.managed_state.get_host_repro_retry_count(),
            )));
        };

        let Some(fw_info) = self.parsed_hosts.find_fw_info_for_host(&explored_endpoint) else {
            return Ok(StateHandlerOutcome::transition(scenario.complete_state()));
        };

        for firmware_type in fw_info.ordering() {
            // ordering() will give a list of firmware types in the order they should be installed.
            // So, `firmware_type` may not be equal to `current_firmware_type` inside this loop.
            // We need to set `firmware_number` to 0 in case they are not equal because `firmware_number` coming
            // from outside this loop belongs only to the `current_firmware_type`
            let firmware_number = if let Some(ft) = current_firmware_type
                && ft == firmware_type
            {
                current_firmware_number
            } else {
                0
            };

            if let Some(to_install) =
                need_host_fw_upgrade(&explored_endpoint, &fw_info, firmware_type)
            {
                if to_install.script.is_some() {
                    return self
                        .by_script(to_install, state, explored_endpoint, scenario)
                        .await;
                }
                tracing::info!(%machine_id,
                    "Installing {:?} (number #{}) on {}",
                    to_install,
                    firmware_number,
                    explored_endpoint.address
                );

                if !repeat && to_install.pre_update_resets {
                    return self
                        .pre_update_resets(state, ctx.services, scenario, None, &None)
                        .await;
                }

                return self
                    .initiate_host_fw_update(
                        explored_endpoint.address,
                        state,
                        ctx,
                        FullFirmwareInfo {
                            model: fw_info.model.as_str(),
                            to_install: &to_install,
                            component_type: &firmware_type,
                            firmware_number: &firmware_number,
                        },
                        scenario,
                    )
                    .await;
            }
        }

        // Nothing needs updates, return to ready.  But first, we may need to reenable lockdown.

        let redfish_client = ctx
            .services
            .create_redfish_client_from_machine(&state.host_snapshot)
            .await?;

        let lockdown_disabled = match redfish_client.lockdown_status().await {
            Ok(status) => !status.is_fully_enabled(), // If it was partial, treat as disabled so we will fully enable it
            Err(e) => {
                if let libredfish::RedfishError::NotSupported(_) = e {
                    // Returned when the platform doesn't support lockdown, so here we say it's not disabled
                    // Note that this is different from the place where we do something similar
                    false
                } else {
                    tracing::warn!("Could not get lockdown status for {machine_id}: {e}",);
                    return Ok(StateHandlerOutcome::do_nothing());
                }
            }
        };
        if lockdown_disabled {
            tracing::debug!("host firmware update: Reenabling lockdown");
            // Already disabled, we need to reenable.
            if let Err(e) = redfish_client
                .lockdown(libredfish::EnabledDisabled::Enabled)
                .await
            {
                tracing::error!("Could not set lockdown for {machine_id}: {e}");
                return Ok(StateHandlerOutcome::do_nothing());
            }
            // Reenabling lockdown will poll lockdown status to verify settings are applied.
            match scenario {
                HostFirmwareScenario::Ready => Ok(StateHandlerOutcome::transition(
                    ManagedHostState::HostInit {
                        machine_state: MachineState::WaitingForLockdown {
                            lockdown_info: LockdownInfo {
                                state: LockdownState::PollingLockdownStatus,
                                mode: Enable,
                            },
                        },
                    },
                )),
                HostFirmwareScenario::Instance => {
                    handler_host_power_control(state, ctx, SystemPowerControl::ForceRestart)
                        .await?;
                    Ok(StateHandlerOutcome::transition(scenario.complete_state()))
                }
            }
        } else {
            tracing::debug!("host firmware update: Don't need to reenable lockdown");
            if let HostFirmwareScenario::Instance = scenario {
                handler_host_power_control(state, ctx, SystemPowerControl::ForceRestart).await?;
            }
            Ok(StateHandlerOutcome::transition(scenario.complete_state()))
        }
    }
}
