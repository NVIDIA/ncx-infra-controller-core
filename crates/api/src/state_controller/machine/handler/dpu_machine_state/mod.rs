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

mod dpu_discovering_state;
mod dpu_init_state;

use std::sync::Arc;

use carbide_uuid::machine::MachineId;
use libredfish::model::task::TaskState;
use libredfish::{Redfish, RedfishError, SystemPowerControl};
use model::machine::{
    DpuDiscoveringState, DpuInitState, InstallDpuOsState, Machine, MachineState, ManagedHostState,
    ManagedHostStateSnapshot, SetSecureBootState,
};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use super::ReachabilityParams;
use super::helpers::{DpuDiscoveringStateHelper, DpuInitStateHelper};
use crate::cfg::file::FirmwareConfig;
use crate::dpf::DpfOperations;
use crate::redfish;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandler, StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

/// A `StateHandler` implementation for DPU machines
#[derive(Debug, Clone)]
pub struct DpuMachineStateHandler {
    pub(super) dpu_nic_firmware_initial_update_enabled: bool,
    pub(super) hardware_models: FirmwareConfig,
    pub(super) reachability_params: ReachabilityParams,
    pub(super) enable_secure_boot: bool,
    pub dpf_sdk: Option<Arc<dyn DpfOperations>>,
}

impl DpuMachineStateHandler {
    pub fn new(
        dpu_nic_firmware_initial_update_enabled: bool,
        hardware_models: FirmwareConfig,
        reachability_params: ReachabilityParams,
        enable_secure_boot: bool,
        dpf_sdk: Option<Arc<dyn DpfOperations>>,
    ) -> Self {
        DpuMachineStateHandler {
            dpu_nic_firmware_initial_update_enabled,
            hardware_models,
            reachability_params,
            enable_secure_boot,
            dpf_sdk,
        }
    }

    async fn is_secure_boot_disabled(
        &self,
        // passing in dpu_machine_id only for testing
        dpu_machine_id: &MachineId,
        dpu_redfish_client: &dyn Redfish,
    ) -> Result<bool, StateHandlerError> {
        let secure_boot_status = dpu_redfish_client.get_secure_boot().await.map_err(|e| {
            StateHandlerError::RedfishError {
                operation: "disable_secure_boot",
                error: e,
            }
        })?;

        let secure_boot_enable =
            secure_boot_status
                .secure_boot_enable
                .ok_or(StateHandlerError::MissingData {
                    object_id: dpu_machine_id.to_string(),
                    missing: "expected secure_boot_enable_field set in secure boot response",
                })?;

        let secure_boot_current_boot =
            secure_boot_status
                .secure_boot_current_boot
                .ok_or(StateHandlerError::MissingData {
                    object_id: dpu_machine_id.to_string(),
                    missing: "expected secure_boot_enable_field set in secure boot response",
                })?;

        Ok(!secure_boot_enable && !secure_boot_current_boot.is_enabled())
    }

    pub(super) async fn handle_dpu_discovering_state(
        &self,
        state: &ManagedHostStateSnapshot,
        dpu_snapshot: &Machine,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        dpu_discovering_state::handle(self, state, dpu_snapshot, ctx).await
    }

    async fn handle_dpuinit_state(
        &self,
        state: &ManagedHostStateSnapshot,
        dpu_snapshot: &Machine,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        dpu_init_state::handle(self, state, dpu_snapshot, ctx).await
    }

    pub(super) async fn set_secure_boot(
        &self,
        count: u32,
        state: &ManagedHostStateSnapshot,
        set_secure_boot_state: SetSecureBootState,
        enable_secure_boot: bool,
        dpu_snapshot: &Machine,
        dpu_redfish_client: &dyn Redfish,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let next_state: ManagedHostState;
        let dpu_machine_id = &dpu_snapshot.id.clone();

        // Use the host snapshot instead of the DPU snapshot because
        // the state.host_snapshot.current.version might be a bit more correct:
        // the state machine is driven by the host state
        let time_since_state_change: chrono::TimeDelta =
            state.host_snapshot.state.version.since_state_change();

        let wait_for_dpu_to_come_up = if time_since_state_change.num_minutes() > 5 {
            false
        } else {
            let (has_dpu_finished_booting, dpu_boot_progress) =
                redfish::did_dpu_finish_booting(dpu_redfish_client)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "did_dpu_finish_booting",
                        error: e,
                    })?;

            if count > 0 && !has_dpu_finished_booting {
                tracing::info!(
                    "Waiting for DPU {} to finish booting; boot progress: {dpu_boot_progress:#?}; SetSecureBoot cycle: {count}",
                    dpu_snapshot.id
                )
            }

            !has_dpu_finished_booting
        };

        match set_secure_boot_state {
            SetSecureBootState::WaitCertificateUpload { task_id } => {
                let task = dpu_redfish_client
                    .get_task(task_id.as_str())
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "get_task",
                        error: e,
                    })?;
                match task.clone().task_state {
                    Some(TaskState::New)
                    | Some(TaskState::Starting)
                    | Some(TaskState::Running)
                    | Some(TaskState::Pending) => {
                        return Ok(StateHandlerOutcome::wait(format!(
                            "Waiting for certificate upload task {task_id} to complete",
                        )));
                    }
                    Some(TaskState::Completed) => {
                        next_state = DpuDiscoveringState::EnableSecureBoot {
                            enable_secure_boot_state: SetSecureBootState::SetSecureBoot,
                            count: 0,
                        }
                        .next_state(&state.managed_state, dpu_machine_id)?;
                    }
                    None => {
                        return Err(StateHandlerError::RedfishError {
                            operation: "get_task",
                            error: RedfishError::NoContent,
                        });
                    }
                    Some(e) => {
                        return Err(StateHandlerError::RedfishError {
                            operation: "get_task",
                            error: RedfishError::GenericError {
                                error: format!("Task {task:#?} error: {e:#?}"),
                            },
                        });
                    }
                }
            }
            SetSecureBootState::CheckSecureBootStatus => {
                // This is the logic:
                // CheckSecureBootStatus -> DisableSecureBoot -> DisableSecureBootState::RebootDPU{0} -> DisableSecureBootState::RebootDPU{1}
                // The first time we check to see if secure boot is disabled, we do not need to wait. The DPU should already be up.
                // However, we need to give time in between the second reboot and checking the status again.
                if count > 0 && wait_for_dpu_to_come_up {
                    return Ok(StateHandlerOutcome::wait(format!(
                        "Waiting for DPU {dpu_machine_id} to come back up from last reboot; time since last reboot: {time_since_state_change}; DisableSecureBoot cycle: {count}",
                    )));
                }

                match self
                    .is_secure_boot_disabled(dpu_machine_id, dpu_redfish_client)
                    .await
                {
                    Ok(is_secure_boot_disabled) if !enable_secure_boot => {
                        if is_secure_boot_disabled {
                            next_state = DpuDiscoveringState::SetUefiHttpBoot
                                .next_state(&state.managed_state, dpu_machine_id)?;
                        } else {
                            next_state = DpuDiscoveringState::DisableSecureBoot {
                                disable_secure_boot_state: Some(SetSecureBootState::SetSecureBoot),
                                count,
                            }
                            .next_state(&state.managed_state, dpu_machine_id)?;
                        }
                    }
                    Ok(is_secure_boot_disabled) => {
                        if is_secure_boot_disabled {
                            let pk_certs = dpu_redfish_client
                                .get_secure_boot_certificates("PK")
                                .await
                                .map_err(|e| StateHandlerError::RedfishError {
                                    operation: "get_secure_boot_certificates",
                                    error: e,
                                })?;

                            if pk_certs.is_empty() {
                                let mut cert_file = File::open("/forge-boot-artifacts/blobs/internal/aarch64/secure-boot-pk.pem").await.map_err(|e| StateHandlerError::RedfishError {
                                    operation: "open_secure_boot_certificate_file",
                                    error: RedfishError::FileError(format!("Error opening secure boot certificate file: {e}")),
                                })?;
                                let mut cert_string = String::new();
                                cert_file
                                    .read_to_string(&mut cert_string)
                                    .await
                                    .map_err(|e| StateHandlerError::RedfishError {
                                        operation: "read_secure_boot_certificate_file",
                                        error: RedfishError::FileError(format!(
                                            "Error reading secure boot certificate file: {e}"
                                        )),
                                    })?;
                                let task = dpu_redfish_client
                                    .add_secure_boot_certificate(cert_string.as_str(), "PK")
                                    .await
                                    .map_err(|e| StateHandlerError::RedfishError {
                                        operation: "add_secure_boot_certificate",
                                        error: e,
                                    })?;
                                dpu_redfish_client
                                    .power(SystemPowerControl::ForceRestart)
                                    .await
                                    .map_err(|e| StateHandlerError::RedfishError {
                                        operation: "force_restart",
                                        error: e,
                                    })?;
                                next_state = DpuDiscoveringState::EnableSecureBoot {
                                    enable_secure_boot_state:
                                        SetSecureBootState::WaitCertificateUpload {
                                            task_id: task.id,
                                        },
                                    count: 0,
                                }
                                .next_state(&state.managed_state, dpu_machine_id)?;
                            } else {
                                next_state = DpuDiscoveringState::EnableSecureBoot {
                                    enable_secure_boot_state: SetSecureBootState::SetSecureBoot,
                                    count,
                                }
                                .next_state(&state.managed_state, dpu_machine_id)?;
                            }
                        } else {
                            next_state = DpuInitState::InstallDpuOs {
                                substate: InstallDpuOsState::InstallingBFB,
                            }
                            .next_state(&state.managed_state, dpu_machine_id)?;
                        }
                    }
                    Err(StateHandlerError::MissingData { object_id, missing }) => {
                        tracing::info!(
                            "Missing data in secure boot status response for DPU {}: {}; rebooting DPU as a work-around",
                            object_id,
                            missing
                        );

                        /***
                         * If the DPU's BMC comes up after UEFI client was run on an ARM
                         * there is a known issue where the redfish query for the secure boot
                         * status comes back incomplete.
                         * Example:
                         * {
                                "@odata.id": "/redfish/v1/Systems/Bluefield/SecureBoot",
                                "@odata.type": "#SecureBoot.v1_1_0.SecureBoot",
                                "Description": "The UEFI Secure Boot associated with this system.",
                                "Id": "SecureBoot",
                                "Name": "UEFI Secure Boot",
                                "SecureBootDatabases": {
                                    "@odata.id": "/redfish/v1/Systems/Bluefield/SecureBoot/SecureBootDatabases"
                            }

                        (missing the SecureBootEnable and SecureBootCurrentBoot fields)
                        The known work around for this issue is to reboot the DPU's ARM. There is a pending FR
                        to fix this on the hardware level.
                        ***/

                        // Do not reboot the DPU indefinitely, something else might be wrong (DPU might be bust).
                        if count < 10 {
                            dpu_redfish_client
                                .power(SystemPowerControl::ForceRestart)
                                .await
                                .map_err(|e| StateHandlerError::RedfishError {
                                    operation: "force_restart",
                                    error: e,
                                })?;
                            if enable_secure_boot {
                                next_state = DpuDiscoveringState::EnableSecureBoot {
                                    enable_secure_boot_state: SetSecureBootState::RebootDPU {
                                        reboot_count: 0,
                                    },
                                    count: count + 1,
                                }
                                .next_state(&state.managed_state, dpu_machine_id)?;
                            } else {
                                next_state = DpuDiscoveringState::DisableSecureBoot {
                                    disable_secure_boot_state: Some(
                                        SetSecureBootState::CheckSecureBootStatus,
                                    ),
                                    count: count + 1,
                                }
                                .next_state(&state.managed_state, dpu_machine_id)?;
                            }
                        } else {
                            return Err(StateHandlerError::MissingData { object_id, missing });
                        }
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            SetSecureBootState::DisableSecureBoot | SetSecureBootState::SetSecureBoot => {
                if enable_secure_boot {
                    dpu_redfish_client.enable_secure_boot().await.map_err(|e| {
                        StateHandlerError::RedfishError {
                            operation: "enable_secure_boot",
                            error: e,
                        }
                    })?;

                    next_state = DpuDiscoveringState::EnableSecureBoot {
                        enable_secure_boot_state: SetSecureBootState::RebootDPU { reboot_count: 0 },
                        count,
                    }
                    .next_state(&state.managed_state, dpu_machine_id)?;
                } else {
                    dpu_redfish_client
                        .disable_secure_boot()
                        .await
                        .map_err(|e| StateHandlerError::RedfishError {
                            operation: "disable_secure_boot",
                            error: e,
                        })?;

                    next_state = DpuDiscoveringState::DisableSecureBoot {
                        disable_secure_boot_state: Some(SetSecureBootState::RebootDPU {
                            reboot_count: 0,
                        }),
                        count,
                    }
                    .next_state(&state.managed_state, dpu_machine_id)?;
                }
            }
            // DPUs requires two reboots after the previous step in order to disable secure boot.
            // From the doc linked above: "the BlueField Arm OS must be rebooted twice. The first
            // reboot is for the UEFI redfish client to read the request from the BMC and apply it; the
            // second reboot is for the setting to take effect."
            // We do not need to wait between disabling secure boot and the first reboot.
            // But, we need to give the DPU time to come up after the initial reboot,
            // before we reboot it again.
            SetSecureBootState::RebootDPU { reboot_count } => {
                if reboot_count == 0 {
                    next_state = if enable_secure_boot {
                        DpuDiscoveringState::EnableSecureBoot {
                            enable_secure_boot_state: SetSecureBootState::RebootDPU {
                                reboot_count: reboot_count + 1,
                            },
                            count,
                        }
                        .next_state(&state.managed_state, dpu_machine_id)?
                    } else {
                        DpuDiscoveringState::DisableSecureBoot {
                            disable_secure_boot_state: Some(SetSecureBootState::RebootDPU {
                                reboot_count: reboot_count + 1,
                            }),
                            count,
                        }
                        .next_state(&state.managed_state, dpu_machine_id)?
                    };
                } else {
                    if wait_for_dpu_to_come_up {
                        return Ok(StateHandlerOutcome::wait(format!(
                            "Waiting for DPU {dpu_machine_id} to come back up from last reboot; time since last reboot: {time_since_state_change}",
                        )));
                    }
                    if enable_secure_boot {
                        next_state = DpuDiscoveringState::EnableSecureBoot {
                            enable_secure_boot_state: SetSecureBootState::CheckSecureBootStatus,
                            count: count + 1,
                        }
                        .next_state(&state.managed_state, dpu_machine_id)?;
                    } else {
                        next_state = DpuDiscoveringState::DisableSecureBoot {
                            disable_secure_boot_state: Some(
                                SetSecureBootState::CheckSecureBootStatus,
                            ),
                            count: count + 1,
                        }
                        .next_state(&state.managed_state, dpu_machine_id)?;
                    }
                }

                dpu_redfish_client
                    .power(SystemPowerControl::ForceRestart)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "force_restart",
                        error: e,
                    })?;
            }
        }

        Ok(StateHandlerOutcome::transition(next_state))
    }
}

#[async_trait::async_trait]
impl StateHandler for DpuMachineStateHandler {
    type State = ManagedHostStateSnapshot;
    type ControllerState = ManagedHostState;
    type ObjectId = MachineId;
    type ContextObjects = MachineStateHandlerContextObjects;

    async fn handle_object_state(
        &self,
        _host_machine_id: &MachineId,
        state: &mut ManagedHostStateSnapshot,
        _controller_state: &Self::ControllerState,
        ctx: &mut StateHandlerContext<Self::ContextObjects>,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        let mut state_handler_outcome = StateHandlerOutcome::do_nothing();
        if state.host_snapshot.associated_dpu_machine_ids().is_empty() {
            let next_state = ManagedHostState::HostInit {
                machine_state: MachineState::WaitingForPlatformConfiguration,
            };
            Ok(StateHandlerOutcome::transition(next_state))
        } else {
            for dpu_snapshot in &state.dpu_snapshots {
                state_handler_outcome = self.handle_dpuinit_state(state, dpu_snapshot, ctx).await?;

                if let outcome @ StateHandlerOutcome::Transition { .. } = state_handler_outcome {
                    return Ok(outcome);
                }
            }

            Ok(state_handler_outcome)
        }
    }
}
