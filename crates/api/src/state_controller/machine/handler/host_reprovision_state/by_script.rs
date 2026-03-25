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
    pub(super) async fn by_script(
        &self,
        to_install: FirmwareEntry,
        state: &ManagedHostStateSnapshot,
        explored_endpoint: model::site_explorer::ExploredEndpoint,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let machine_id = state.host_snapshot.id;

        self.upgrade_script_state.started(machine_id.to_string());

        let address = explored_endpoint.address.to_string().clone();
        let script = to_install.script.unwrap_or("/bin/false".into()); // Should always be Some at this point
        let upgrade_script_state = self.upgrade_script_state.clone();
        let (username, password) = if let Some(credential_reader) = &self.credential_reader {
            let bmc_mac_address =
                state
                    .host_snapshot
                    .bmc_info
                    .mac
                    .ok_or_else(|| StateHandlerError::MissingData {
                        object_id: state.host_snapshot.id.to_string(),
                        missing: "bmc_mac",
                    })?;
            let key = CredentialKey::BmcCredentials {
                credential_type: BmcCredentialType::BmcRoot { bmc_mac_address },
            };
            match credential_reader.get_credentials(&key).await {
                Ok(Some(credentials)) => match credentials {
                    Credentials::UsernamePassword { username, password } => (username, password),
                },
                Ok(None) => {
                    return Err(StateHandlerError::GenericError(eyre!(
                        "No BMC credentials exists"
                    )));
                }
                Err(e) => {
                    return Err(StateHandlerError::GenericError(eyre!(
                        "Unable to get BMC credentials: {e}"
                    )));
                }
            }
        } else {
            ("Unknown".to_string(), "Unknown".to_string())
        };
        tokio::spawn(async move {
            let mut cmd = match tokio::process::Command::new(script)
                .env("BMC_IP", address.clone())
                .env("BMC_USERNAME", username)
                .env("BMC_PASSWORD", password)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
            {
                Ok(cmd) => cmd,
                Err(e) => {
                    tracing::error!(
                        "Upgrade script {machine_id} {address} command creation failed: {e}"
                    );
                    upgrade_script_state.completed(machine_id.to_string(), false);
                    return;
                }
            };

            let Some(stdout) = cmd.stdout.take() else {
                tracing::error!("Upgrade script {machine_id} {address} STDOUT creation failed");
                let _ = cmd.kill().await;
                let _ = cmd.wait().await;
                upgrade_script_state.completed(machine_id.to_string(), false);
                return;
            };
            let stdout = tokio::io::BufReader::new(stdout);

            let Some(stderr) = cmd.stderr.take() else {
                tracing::error!("Upgrade script {machine_id} {address} STDERR creation failed");
                let _ = cmd.kill().await;
                let _ = cmd.wait().await;
                upgrade_script_state.completed(machine_id.to_string(), false);
                return;
            };
            let stderr = tokio::io::BufReader::new(stderr);

            // Take the stdout and stderr from the script and write them to a log with a searchable prefix
            let machine_id2 = address.clone();
            let address2 = address.clone();
            let thread = tokio::spawn(async move {
                let mut lines = stderr.lines();
                while let Some(line) = lines.next_line().await.unwrap_or(None) {
                    tracing::info!("Upgrade script {machine_id2} {address2} STDERR {line}");
                }
            });
            let mut lines = stdout.lines();
            while let Some(line) = lines.next_line().await.unwrap_or(None) {
                tracing::info!("Upgrade script {machine_id} {address} {line}");
            }
            let _ = tokio::join!(thread);

            match cmd.wait().await {
                Err(e) => {
                    tracing::info!(
                        "Upgrade script {machine_id} {address} FAILED: Wait failure {e}"
                    );
                    upgrade_script_state.completed(machine_id.to_string(), false);
                }
                Ok(errorcode) => {
                    if errorcode.success() {
                        tracing::info!(
                            "Upgrade script {machine_id} {address} completed successfully"
                        );
                        upgrade_script_state.completed(machine_id.to_string(), true);
                    } else {
                        tracing::warn!(
                            "Upgrade script {machine_id} {address} FAILED: Exited with {errorcode}"
                        );
                        upgrade_script_state.completed(machine_id.to_string(), false);
                    }
                }
            }
        });

        Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
            HostReprovisionState::WaitingForScript {},
            state.managed_state.get_host_repro_retry_count(),
        )))
    }

    pub(super) fn waiting_for_manual_upgrade(
        &self,
        state: &ManagedHostStateSnapshot,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let machine_id = &state.host_snapshot.id;

        if let Some(completed_at) = state.host_snapshot.manual_firmware_upgrade_completed {
            tracing::info!(
                "Manual firmware upgrade completed for {} at {}, proceeding to automatic upgrades",
                machine_id,
                completed_at
            );

            return Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_type: None,
                    firmware_number: None,
                },
                state.managed_state.get_host_repro_retry_count(),
            )));
        }

        tracing::debug!(
            "Machine {} still waiting for manual firmware upgrade to be marked complete",
            machine_id
        );
        Ok(StateHandlerOutcome::do_nothing())
    }

    pub(super) fn waiting_for_script(
        &self,
        state: &ManagedHostStateSnapshot,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let machine_id = state.host_snapshot.id.to_string();
        let Some(success) = self.upgrade_script_state.state(&machine_id) else {
            // Not yet completed, or we restarted (which specifically needs a manual restart of interrupted scripts)
            return Ok(StateHandlerOutcome::do_nothing());
        };

        self.upgrade_script_state.clear(&machine_id);

        if success {
            Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_type: None,
                    firmware_number: None,
                },
                state.managed_state.get_host_repro_retry_count(),
            )))
        } else {
            let reprovision_state = HostReprovisionState::FailedFirmwareUpgrade {
                firmware_type: FirmwareComponentType::Unknown,
                report_time: Some(Utc::now()),
                reason: Some(format!(
                    "The upgrade script failed.  Search the log for \"Upgrade script {}\" for script output.  Use \"forge-admin-cli mh reset-host-reprovisioning --machine {}\" to retry.",
                    state.host_snapshot.id, state.host_snapshot.id
                )),
            };
            Ok(StateHandlerOutcome::transition(scenario.actual_new_state(
                reprovision_state,
                state.managed_state.get_host_repro_retry_count(),
            )))
        }
    }
}
