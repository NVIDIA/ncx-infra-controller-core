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
    pub(super) async fn host_reset_for_new_firmware(
        &self,
        state: &ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
        machine_id: &MachineId,
        details: &HostReprovisionState,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let (
            final_version,
            firmware_type,
            firmware_number,
            power_drains_needed,
            delay_until,
            last_power_drain_operation,
        ) = match details {
            HostReprovisionState::ResetForNewFirmware {
                final_version,
                firmware_type,
                firmware_number,
                power_drains_needed,
                delay_until,
                last_power_drain_operation,
            } => (
                final_version,
                firmware_type,
                firmware_number,
                power_drains_needed,
                delay_until,
                last_power_drain_operation,
            ),
            _ => {
                return Err(StateHandlerError::GenericError(eyre!(
                    "Wrong enum in host_reset_for_new_firmware"
                )));
            }
        };

        let Some(endpoint) = find_explored_refreshed_endpoint(state, machine_id, ctx).await? else {
            tracing::debug!("Waiting for site explorer to revisit {machine_id}");
            return Ok(StateHandlerOutcome::do_nothing());
        };

        if let Some(power_drains_needed) = power_drains_needed {
            if let Some(delay_until) = delay_until
                && *delay_until > chrono::Utc::now().timestamp()
            {
                tracing::info!(
                    "Waiting after {last_power_drain_operation:?} of {}",
                    &endpoint.address
                );
                return Ok(StateHandlerOutcome::do_nothing());
            }

            match last_power_drain_operation {
                None | Some(PowerDrainState::On) => {
                    // The 1000 is for unit tests; values above this will skip delays.
                    if *power_drains_needed == 0 || *power_drains_needed == 1000 {
                        tracing::info!("Power drains for {} done", &endpoint.address);
                        // This path, and only this path of the match, exits the match and lets us proceed.  All others should return after updating state.
                    } else {
                        tracing::info!(
                            "Upgrade task has completed for {} but needs {} power drain(s), initiating one",
                            &endpoint.address,
                            power_drains_needed
                        );
                        handler_host_power_control(state, ctx, SystemPowerControl::ForceOff)
                            .await?;

                        // Wait 60 seconds after powering off to do AC powercycle
                        let delay = if *power_drains_needed < 1000 { 60 } else { 0 };
                        let reprovision_state = HostReprovisionState::ResetForNewFirmware {
                            final_version: final_version.clone(),
                            firmware_type: *firmware_type,
                            firmware_number: *firmware_number,
                            power_drains_needed: Some(*power_drains_needed),
                            delay_until: Some(chrono::Utc::now().timestamp() + delay),
                            last_power_drain_operation: Some(PowerDrainState::Off),
                        };
                        return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                            reprovision_state,
                            state.managed_state.get_host_repro_retry_count(),
                        )));
                    }
                }
                Some(PowerDrainState::Off) => {
                    tracing::info!("Doing powercycle now for {}", &endpoint.address);
                    handler_host_power_control(state, ctx, SystemPowerControl::ACPowercycle)
                        .await?;

                    let delay = if *power_drains_needed < 1000 { 90 } else { 0 };
                    let reprovision_state = HostReprovisionState::ResetForNewFirmware {
                        final_version: final_version.clone(),
                        firmware_type: *firmware_type,
                        firmware_number: *firmware_number,
                        power_drains_needed: Some(*power_drains_needed),
                        delay_until: Some(chrono::Utc::now().timestamp() + delay),
                        last_power_drain_operation: Some(PowerDrainState::Powercycle),
                    };
                    return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                        reprovision_state,
                        state.managed_state.get_host_repro_retry_count(),
                    )));
                }
                Some(PowerDrainState::Powercycle) => {
                    tracing::info!("Turning back on {}", &endpoint.address);
                    handler_host_power_control(state, ctx, SystemPowerControl::On).await?;

                    let delay = if *power_drains_needed < 1000 { 5 } else { 0 };
                    let reprovision_state = HostReprovisionState::ResetForNewFirmware {
                        final_version: final_version.clone(),
                        firmware_type: *firmware_type,
                        firmware_number: *firmware_number,
                        power_drains_needed: Some(power_drains_needed - 1),
                        delay_until: Some(chrono::Utc::now().timestamp() + delay),
                        last_power_drain_operation: Some(PowerDrainState::On),
                    };
                    return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                        reprovision_state,
                        state.managed_state.get_host_repro_retry_count(),
                    )));
                }
            };
        } else if firmware_type.is_uefi() {
            tracing::debug!(
                "Upgrade task has completed for {} but needs reboot, initiating one",
                &endpoint.address
            );
            handler_host_power_control(state, ctx, SystemPowerControl::ForceRestart).await?;

            // Same state but with the rebooted flag set, it can take a long time to reboot in some cases so we do not retry.
        }

        if firmware_type.is_bmc()
            && !endpoint
                .report
                .vendor
                .unwrap_or(bmc_vendor::BMCVendor::Unknown)
                .is_dell()
        {
            tracing::debug!(
                "Upgrade task has completed for {} but needs BMC reboot, initiating one",
                &endpoint.address
            );
            let redfish_client = ctx
                .services
                .create_redfish_client_from_machine(&state.host_snapshot)
                .await?;

            if let Err(e) = redfish_client.bmc_reset().await {
                tracing::warn!("Failed to reboot {}: {e}", &endpoint.address);
                return Ok(StateHandlerOutcome::do_nothing());
            }
        }

        if (*firmware_type == FirmwareComponentType::HGXBmc
            || *firmware_type == FirmwareComponentType::Gpu)
            && !power_drains_needed.is_some()
        {
            // Needs a host power reset.  We might also have used the power drains to do an AC powercycle.
            let redfish_client = ctx
                .services
                .create_redfish_client_from_machine(&state.host_snapshot)
                .await?;

            // We previously possibly tried to use ACPowerycle here, however that requires enough time for the BMC to come back.  We use
            // the power_drains_needed setting instead for that which is already aware of how to keep track of that sort of thing.
            if let Err(e) = redfish_client.power(SystemPowerControl::ForceOff).await {
                tracing::error!("Failed to power off {}: {e}", &endpoint.address);
                return Ok(StateHandlerOutcome::do_nothing());
            }
            tokio::time::sleep(self.hgx_bmc_gpu_reboot_delay).await;
            if let Err(e) = redfish_client.power(SystemPowerControl::On).await {
                tracing::error!("Failed to power on {}: {e}", &endpoint.address);
                return Ok(StateHandlerOutcome::do_nothing());
            }
            // Okay to proceed
        }

        // Now we can go on to waiting for the correct version to be reported
        let reprovision_state = HostReprovisionState::NewFirmwareReportedWait {
            firmware_type: *firmware_type,
            firmware_number: *firmware_number,
            final_version: final_version.to_string(),
            previous_reset_time: Some(Utc::now().timestamp()),
        };
        Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
            reprovision_state,
            state.managed_state.get_host_repro_retry_count(),
        )))
    }

    pub(super) async fn host_new_firmware_reported_wait(
        &self,
        state: &ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
        details: &HostReprovisionState,
        machine_id: &MachineId,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let (final_version, firmware_type, firmware_number, previous_reset_time) = match details {
            HostReprovisionState::NewFirmwareReportedWait {
                final_version,
                firmware_type,
                firmware_number,
                previous_reset_time,
            } => (
                final_version,
                firmware_type,
                firmware_number,
                previous_reset_time,
            ),
            _ => {
                return Err(StateHandlerError::GenericError(eyre!(
                    "Wrong enum in host_new_firmware_reported_wait"
                )));
            }
        };

        let Some(endpoint) = find_explored_refreshed_endpoint(state, machine_id, ctx).await? else {
            tracing::debug!("Waiting for site explorer to revisit {machine_id}");
            return Ok(StateHandlerOutcome::do_nothing());
        };

        let Some(fw_info) = self.parsed_hosts.find_fw_info_for_host(&endpoint) else {
            tracing::error!("Could no longer find firmware info for {machine_id}");
            return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_type: Some(*firmware_type),
                    firmware_number: *firmware_number,
                },
                state.managed_state.get_host_repro_retry_count(),
            )));
        };

        let current_versions = endpoint.find_all_versions(&fw_info, *firmware_type);
        if current_versions.is_empty() {
            tracing::error!("Could no longer find current versions for {machine_id}");
            return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_type: Some(*firmware_type),
                    firmware_number: *firmware_number,
                },
                state.managed_state.get_host_repro_retry_count(),
            )));
        };

        let versions_match_final_version = current_versions.iter().all(|v| *v == final_version);
        if !versions_match_final_version {
            tracing::warn!(
                "{}: Not all firmware versions match. Expected: {final_version}, Found: {:?}",
                endpoint.address,
                current_versions
            );
        }

        if versions_match_final_version {
            // Done waiting, go back to overall checking of version`2s
            tracing::debug!("Done waiting for {machine_id} to reach version");
            Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_type: Some(*firmware_type),
                    firmware_number: *firmware_number,
                },
                state.managed_state.get_host_repro_retry_count(),
            )))
        } else {
            if !self.no_firmware_update_reset_retries
                && let Some(previous_reset_time) = previous_reset_time
                && previous_reset_time + 30 * 60 <= Utc::now().timestamp()
            {
                tracing::info!(
                    "Upgrade for {} {:?} has taken more than 30 minutes to report new version; resetting again.",
                    &endpoint.address,
                    firmware_type
                );
                let details = &HostReprovisionState::ResetForNewFirmware {
                    final_version: final_version.to_string(),
                    firmware_type: *firmware_type,
                    firmware_number: *firmware_number,
                    power_drains_needed: None,
                    delay_until: None,
                    last_power_drain_operation: None,
                };
                return self
                    .host_reset_for_new_firmware(state, ctx, machine_id, details, scenario)
                    .await;
            }
            tracing::info!(
                "Waiting for {machine_id} {firmware_type:?} to reach version {final_version} currently {current_versions:?}"
            );

            let mut txn = ctx.services.db_pool.begin().await?;
            db::explored_endpoints::re_explore_if_version_matches(
                endpoint.address,
                endpoint.report_version,
                &mut txn,
            )
            .await?;
            Ok(StateHandlerOutcome::do_nothing().with_txn(txn))
        }
    }
}
