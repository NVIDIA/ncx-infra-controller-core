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
    pub(super) async fn pre_update_resets(
        &self,
        state: &ManagedHostStateSnapshot,
        services: &CommonStateHandlerServices,
        scenario: HostFirmwareScenario,
        phase: Option<InitialResetPhase>,
        last_time: &Option<DateTime<Utc>>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let redfish_client = services
            .create_redfish_client_from_machine(&state.host_snapshot)
            .await?;

        match phase.unwrap_or(InitialResetPhase::Start) {
            InitialResetPhase::Start => {
                redfish_client
                    .power(SystemPowerControl::ForceOff)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "power off",
                        error: e,
                    })?;
                let status = redfish_client.get_power_state().await.map_err(|e| {
                    StateHandlerError::RedfishError {
                        operation: "get power state",
                        error: e,
                    }
                })?;
                if status != PowerState::Off {
                    return Err(StateHandlerError::GenericError(eyre!(
                        "Host {} did not turn off when requested",
                        state.host_snapshot.id
                    )));
                }
                redfish_client
                    .bmc_reset()
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "BMC reset",
                        error: e,
                    })?;

                Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                    HostReprovisionState::InitialReset {
                        phase: InitialResetPhase::BMCWasReset,
                        last_time: Utc::now(),
                    },
                    state.managed_state.get_host_repro_retry_count(),
                )))
            }
            InitialResetPhase::BMCWasReset => {
                if let Err(_e) = redfish_client.get_tasks().await {
                    // BMC not fully up yet
                    return Ok(StateHandlerOutcome::do_nothing());
                }
                redfish_client
                    .power(SystemPowerControl::On)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "power on",
                        error: e,
                    })?;
                let status = redfish_client.get_power_state().await.map_err(|e| {
                    StateHandlerError::RedfishError {
                        operation: "get power state",
                        error: e,
                    }
                })?;
                if status != PowerState::On {
                    return Err(StateHandlerError::GenericError(eyre!(
                        "Host {} did not turn on when requested",
                        state.host_snapshot.id
                    )));
                }
                Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                    HostReprovisionState::InitialReset {
                        phase: InitialResetPhase::WaitHostBoot,
                        last_time: Utc::now(),
                    },
                    state.managed_state.get_host_repro_retry_count(),
                )))
            }
            InitialResetPhase::WaitHostBoot => {
                if Utc::now().signed_duration_since(last_time.unwrap_or(Utc::now()))
                    < chrono::TimeDelta::minutes(20)
                {
                    // Wait longer
                    return Ok(StateHandlerOutcome::do_nothing());
                }
                // Now we can actually proceed with the upgrade.  Go back to checking firmware so we don't have to store all of that info.
                Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                    HostReprovisionState::CheckingFirmwareRepeatV2 {
                        firmware_type: None,
                        firmware_number: None,
                    },
                    state.managed_state.get_host_repro_retry_count(),
                )))
            }
        }
    }
}
