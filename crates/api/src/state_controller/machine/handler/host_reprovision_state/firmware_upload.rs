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
    /// Uploads a firmware update via multipart, returning the task ID, or None if upload was deferred
    pub(super) async fn initiate_host_fw_update(
        &self,
        address: std::net::IpAddr,
        state: &ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
        fw_info: FullFirmwareInfo<'_>,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let snapshot = &state.host_snapshot;
        let to_install = fw_info.to_install;
        let component_type = fw_info.component_type;

        if !self.downloader.available(
            &to_install.get_filename(*fw_info.firmware_number),
            &to_install.get_url(),
            &to_install.get_checksum(),
        ) {
            tracing::debug!(
                "{} is being downloaded from {}, update deferred",
                to_install.get_filename(*fw_info.firmware_number).display(),
                to_install.get_url()
            );

            return Ok(StateHandlerOutcome::do_nothing());
        }

        let Ok(_active) = self.upload_limiter.try_acquire() else {
            tracing::debug!(
                "Deferring installation of {:?} on {}, too many uploads already active",
                to_install,
                snapshot.id,
            );
            return Ok(StateHandlerOutcome::do_nothing());
        };

        // Setup the Redfish connection
        let redfish_client = ctx
            .services
            .create_redfish_client_from_machine(snapshot)
            .await?;

        let lockdown_disabled = match redfish_client.lockdown_status().await {
            Ok(status) => status.is_fully_disabled(), // If we're partial, we want to act like it was enabled so we disable it
            Err(e) => {
                if let libredfish::RedfishError::NotSupported(_) = e {
                    // Returned when the platform doesn't support lockdown, so here we say it's already disabled
                    // Note that this is different from the place where we do something similar
                    true
                } else {
                    tracing::warn!(
                        "Could not get lockdown status for {}: {e}",
                        state.host_snapshot.id
                    );
                    return Ok(StateHandlerOutcome::do_nothing());
                }
            }
        };
        if lockdown_disabled {
            // Already disabled, we can go ahead
            tracing::debug!("Host fw update: No need for disabling lockdown");
        } else {
            tracing::info!(%address, "Host fw update: Disabling lockdown");
            if let Err(e) = redfish_client
                .lockdown(libredfish::EnabledDisabled::Disabled)
                .await
            {
                tracing::warn!("Could not set lockdown for {}: {e}", address.to_string());
                return Ok(StateHandlerOutcome::do_nothing());
            }
            if fw_info.model == "Dell" {
                tracing::info!(%address, "Host fw update: Rebooting after disabling lockdown because Dell");
                handler_host_power_control(state, ctx, SystemPowerControl::ForceRestart).await?;
                // Wait until the next state machine iteration to let it restart
                return Ok(StateHandlerOutcome::do_nothing());
            }
        }

        let machine_id = state.host_snapshot.id.to_string();
        let filename = to_install.get_filename(*fw_info.firmware_number);
        let redfish_component_type: libredfish::model::update_service::ComponentType =
            match to_install.install_only_specified {
                false => libredfish::model::update_service::ComponentType::Unknown,
                true => (*component_type).into(),
            };
        let address = address.to_string();

        self.async_firmware_uploader.start_upload(
            machine_id,
            redfish_client,
            filename,
            redfish_component_type,
            address,
        );

        // Upload complete and updated started, will monitor task in future iterations
        let reprovision_state = HostReprovisionState::WaitingForUpload {
            firmware_type: *fw_info.component_type,
            final_version: fw_info.to_install.version.clone(),
            power_drains_needed: fw_info.to_install.power_drains_needed,
            firmware_number: Some(*fw_info.firmware_number),
        };

        Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
            reprovision_state,
            state.managed_state.get_host_repro_retry_count(),
        )))
    }

    pub(super) async fn waiting_for_upload(
        &self,
        details: &HostReprovisionState,
        state: &ManagedHostStateSnapshot,
        scenario: HostFirmwareScenario,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let (final_version, firmware_type, power_drains_needed, firmware_number) = match details {
            HostReprovisionState::WaitingForUpload {
                final_version,
                firmware_type,
                power_drains_needed,
                firmware_number,
            } => (
                final_version,
                firmware_type,
                power_drains_needed,
                firmware_number,
            ),
            _ => {
                return Err(StateHandlerError::GenericError(eyre!(
                    "Wrong enum in waiting_for_upload"
                )));
            }
        };

        let machine_id = state.host_snapshot.id;
        let address = match find_explored_refreshed_endpoint(state, &machine_id, ctx).await {
            Ok(explored_endpoint) => match explored_endpoint {
                Some(explored_endpoint) => explored_endpoint.address.to_string(),
                None => "unknown".to_string(),
            },
            Err(_) => "unknown".to_string(),
        };
        let machine_id = machine_id.to_string();
        match self.async_firmware_uploader.upload_status(&machine_id) {
            None => {
                tracing::info!(
                    "Apparent restart before upload to {machine_id} {address} completion, returning to CheckingFirmware"
                );
                Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                    HostReprovisionState::CheckingFirmwareRepeatV2 {
                        firmware_type: Some(*firmware_type),
                        firmware_number: *firmware_number,
                    },
                    state.managed_state.get_host_repro_retry_count(),
                )))
            }
            Some(upload_status) => {
                match upload_status {
                    None => {
                        tracing::debug!("Upload to {machine_id} {address} not yet complete");
                        Ok(StateHandlerOutcome::do_nothing())
                    }
                    Some(result) => {
                        match result {
                            UploadResult::Success { task_id } => {
                                // We want to remove the machine ID from the hashmap, but do not do it here, because we may fail the commit.  Run it in the next state handling.  Failure case doesn't matter, it would have identical behavior.
                                tracing::info!(
                                    "Upload to {machine_id} {address} completed with task ID {task_id}"
                                );
                                // Upload complete and updated started, will monitor task in future iterations
                                let reprovision_state =
                                    HostReprovisionState::WaitingForFirmwareUpgrade {
                                        task_id,
                                        firmware_type: *firmware_type,
                                        final_version: final_version.clone(),
                                        power_drains_needed: *power_drains_needed,
                                        firmware_number: *firmware_number,
                                        started_waiting: Some(Utc::now()),
                                    };
                                Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                                    reprovision_state,
                                    state.managed_state.get_host_repro_retry_count(),
                                )))
                            }
                            UploadResult::Failure => {
                                self.async_firmware_uploader.finish_upload(&machine_id);
                                // The upload thread already logged this
                                Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                                    HostReprovisionState::CheckingFirmwareRepeatV2 {
                                        firmware_type: Some(*firmware_type),
                                        firmware_number: *firmware_number,
                                    },
                                    state.managed_state.get_host_repro_retry_count(),
                                )))
                            }
                        }
                    }
                }
            }
        }
    }
}
