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
    pub(super) async fn host_waiting_fw(
        &self,
        details: &HostReprovisionState,
        state: &ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
        machine_id: &MachineId,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let (
            task_id,
            final_version,
            firmware_type,
            power_drains_needed,
            firmware_number,
            started_waiting,
        ) = match details {
            HostReprovisionState::WaitingForFirmwareUpgrade {
                task_id,
                final_version,
                firmware_type,
                power_drains_needed,
                firmware_number,
                started_waiting,
            } => (
                task_id,
                final_version,
                firmware_type,
                power_drains_needed,
                firmware_number,
                started_waiting,
            ),
            _ => {
                return Err(StateHandlerError::GenericError(eyre!(
                    "Wrong enum in host_waiting_fw"
                )));
            }
        };

        // Now it's safe to clear the hashmap for the upload status
        self.async_firmware_uploader
            .finish_upload(&state.host_snapshot.id.to_string());

        let address = state
            .host_snapshot
            .bmc_info
            .ip_addr()
            .map_err(StateHandlerError::GenericError)?;
        // Setup the Redfish connection
        let redfish_client = ctx
            .services
            .create_redfish_client_from_machine(&state.host_snapshot)
            .await?;

        match redfish_client.get_task(task_id.as_str()).await {
            Ok(task_info) => {
                match task_info.task_state {
                    Some(TaskState::New)
                    | Some(TaskState::Starting)
                    | Some(TaskState::Running)
                    | Some(TaskState::Pending) => {
                        tracing::debug!(
                            "Upgrade task for {} not yet complete, current state {:?} message {:?}",
                            machine_id,
                            task_info.task_state,
                            task_info.messages,
                        );
                        Ok(StateHandlerOutcome::do_nothing())
                    }
                    Some(TaskState::Completed) => {
                        // Task has completed, update is done and we can clean up.  Site explorer will ingest this next time it runs on this endpoint.

                        // If we have multiple firmware files to be uploaded, do the next one.
                        if let Some(endpoint) =
                            find_explored_refreshed_endpoint(state, machine_id, ctx).await?
                            && let Some(fw_info) =
                                self.parsed_hosts.find_fw_info_for_host(&endpoint)
                            && let Some(component_info) = fw_info.components.get(firmware_type)
                            && let Some(selected_firmware) =
                                component_info.known_firmware.iter().find(|&x| x.default)
                        {
                            let firmware_number = firmware_number.unwrap_or(0) + 1;
                            if firmware_number
                                < selected_firmware.filenames.len().try_into().unwrap_or(0)
                            {
                                tracing::debug!(
                                    "Moving {:?} chain step {} on {} to CheckingFirmware",
                                    selected_firmware,
                                    firmware_number,
                                    endpoint.address
                                );

                                // There are more files to install.
                                // Move to CheckingFirmware and start installing
                                let reprovision_state = HostReprovisionState::CheckingFirmwareV2 {
                                    firmware_type: Some(*firmware_type),
                                    firmware_number: Some(firmware_number),
                                };

                                return Ok(StateHandlerOutcome::transition(
                                    scenario.actual_new_state(
                                        reprovision_state,
                                        state.managed_state.get_host_repro_retry_count(),
                                    ),
                                ));
                            }
                        }

                        tracing::debug!(
                            "Saw completion of host firmware upgrade task for {}",
                            machine_id
                        );

                        let reprovision_state = HostReprovisionState::ResetForNewFirmware {
                            final_version: final_version.to_string(),
                            firmware_type: *firmware_type,
                            firmware_number: *firmware_number,
                            power_drains_needed: *power_drains_needed,
                            delay_until: None,
                            last_power_drain_operation: None,
                        };
                        Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                            reprovision_state,
                            state.managed_state.get_host_repro_retry_count(),
                        )))
                    }
                    Some(TaskState::Exception)
                    | Some(TaskState::Interrupted)
                    | Some(TaskState::Killed)
                    | Some(TaskState::Cancelled) => {
                        let msg = format!(
                            "Failure in firmware upgrade for {}: {} {:?}",
                            machine_id,
                            task_info.task_state.unwrap(),
                            task_info
                                .messages
                                .last()
                                .map_or("".to_string(), |m| m.message.clone())
                        );
                        tracing::warn!(msg);

                        // We need site explorer to requery the version, just in case it actually did get done
                        let mut txn = ctx.services.db_pool.begin().await?;

                        db::explored_endpoints::set_waiting_for_explorer_refresh(address, &mut txn)
                            .await?;

                        Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                            HostReprovisionState::FailedFirmwareUpgrade {
                                firmware_type: *firmware_type,
                                report_time: Some(Utc::now()),
                                reason: Some(msg),
                            },
                            state.managed_state.get_host_repro_retry_count(),
                        ))
                        .with_txn(txn))
                    }
                    _ => {
                        // Unexpected state
                        let msg = format!(
                            "Unrecognized task state for {}: {:?}",
                            machine_id, task_info.task_state
                        );
                        tracing::warn!(msg);

                        let reprovision_state = HostReprovisionState::FailedFirmwareUpgrade {
                            firmware_type: *firmware_type,
                            report_time: Some(Utc::now()),
                            reason: Some(msg),
                        };
                        Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                            reprovision_state,
                            state.managed_state.get_host_repro_retry_count(),
                        )))
                    }
                }
            }
            Err(e) => match e {
                RedfishError::HTTPErrorCode { status_code, .. } => {
                    if status_code == NOT_FOUND {
                        // Dells (maybe others) have been observed to not have report the job any more after completing a host reboot for a UEFI upgrade.  If we get a 404 but see that we're at the right version, we're done with that upgrade.
                        let Some(endpoint) =
                            find_explored_refreshed_endpoint(state, machine_id, ctx).await?
                        else {
                            return Ok(StateHandlerOutcome::do_nothing());
                        };

                        if let Some(fw_info) = self.parsed_hosts.find_fw_info_for_host(&endpoint)
                            && let Some(current_version) =
                                endpoint.find_version(&fw_info, *firmware_type)
                            && current_version == final_version
                        {
                            tracing::info!(
                                "Marking completion of Redfish task of firmware upgrade for {} with missing task",
                                &endpoint.address
                            );

                            return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                                HostReprovisionState::CheckingFirmwareRepeatV2 {
                                    firmware_type: Some(*firmware_type),
                                    firmware_number: *firmware_number,
                                },
                                state.managed_state.get_host_repro_retry_count(),
                            )));
                        }

                        // We have also observed (FORGE-6177) the upgrade somehow disappearing, but working when retried.  If a long time has passed, go back to checking to retry.
                        if let Some(started_waiting) = started_waiting
                            && Utc::now().signed_duration_since(started_waiting)
                                > chrono::TimeDelta::minutes(15)
                        {
                            tracing::info!(%machine_id,
                                "Timed out with missing Redfish task for firmware upgrade for {}, returning to CheckingFirmware",
                                &endpoint.address
                            );
                            return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                                HostReprovisionState::CheckingFirmwareRepeatV2 {
                                    firmware_type: Some(*firmware_type),
                                    firmware_number: *firmware_number,
                                },
                                state.managed_state.get_host_repro_retry_count(),
                            )));
                        }
                    }
                    Err(StateHandlerError::RedfishError {
                        operation: "get_task",
                        error: e,
                    })
                }
                _ => Err(StateHandlerError::RedfishError {
                    operation: "get_task",
                    error: e,
                }),
            },
        }
    }
}
