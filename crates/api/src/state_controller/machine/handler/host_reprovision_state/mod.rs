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

mod by_script;
mod checking_firmware;
mod firmware_reset;
mod firmware_upload;
mod firmware_wait;
mod pre_update_resets;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use carbide_uuid::machine::MachineId;
use chrono::{DateTime, Utc};
use db::DatabaseError;
use eyre::eyre;
use forge_secrets::credentials::{BmcCredentialType, CredentialKey, CredentialReader, Credentials};
use futures_util::FutureExt;
use libredfish::model::task::TaskState;
use libredfish::{PowerState, Redfish, RedfishError, SystemPowerControl};
use model::firmware::{FirmwareComponentType, FirmwareEntry};
use model::machine::LockdownMode::Enable;
use model::machine::{
    HostReprovisionState, InitialResetPhase, InstanceState, LockdownInfo, LockdownState,
    MachineState, ManagedHostState, ManagedHostStateSnapshot, PowerDrainState,
};
use tokio::io::AsyncBufReadExt;
use tokio::sync::Semaphore;

use super::{
    FullFirmwareInfo, MAX_FIRMWARE_UPGRADE_RETRIES, NOT_FOUND, find_explored_refreshed_endpoint,
    handler_host_power_control, need_host_fw_upgrade, requires_manual_firmware_upgrade,
};
use crate::cfg::file::{FirmwareConfig, TimePeriod};
use crate::firmware_downloader::FirmwareDownloader;
use crate::state_controller::common_services::CommonStateHandlerServices;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

#[derive(Debug)]
pub(super) enum HostFirmwareScenario {
    Ready,
    Instance,
}

impl HostFirmwareScenario {
    fn actual_new_state(
        &self,
        reprovision_state: HostReprovisionState,
        host_retry_count: u32,
    ) -> ManagedHostState {
        match self {
            HostFirmwareScenario::Ready => ManagedHostState::HostReprovision {
                reprovision_state,
                retry_count: host_retry_count,
            },
            HostFirmwareScenario::Instance => ManagedHostState::Assigned {
                instance_state: InstanceState::HostReprovision { reprovision_state },
            },
        }
    }

    fn complete_state(&self) -> ManagedHostState {
        match self {
            HostFirmwareScenario::Ready => ManagedHostState::Ready,
            HostFirmwareScenario::Instance => ManagedHostState::Assigned {
                instance_state: InstanceState::Ready,
            },
        }
    }
}

#[derive(Debug, Clone)]
enum UploadResult {
    Success { task_id: String },
    Failure,
}

pub(super) struct HostUpgradeState {
    pub(super) parsed_hosts: Arc<FirmwareConfig>,
    pub(super) downloader: FirmwareDownloader,
    pub(super) upload_limiter: Arc<Semaphore>,
    pub(super) no_firmware_update_reset_retries: bool,
    pub(super) instance_autoreboot_period: Option<TimePeriod>,
    pub(super) upgrade_script_state: Arc<UpdateScriptManager>,
    pub(super) credential_reader: Option<Arc<dyn CredentialReader>>,
    pub(super) async_firmware_uploader: Arc<AsyncFirmwareUploader>,
    pub(super) hgx_bmc_gpu_reboot_delay: tokio::time::Duration,
}

impl std::fmt::Debug for HostUpgradeState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "HostUpgradeState: parsed_hosts: {:?} downloader: {:?} upload_limiter: {:?} no_firmware_update_reset_retries: {:?} instance_autoreboot_period: {:?}, upgrade_script_state: {:?}",
            self.parsed_hosts,
            self.downloader,
            self.upload_limiter,
            self.no_firmware_update_reset_retries,
            self.instance_autoreboot_period,
            self.upgrade_script_state
        )
    }
}

impl HostUpgradeState {
    // Handles when in HostReprovisioning or when entering it
    pub(super) async fn handle_host_reprovision(
        &self,
        state: &mut ManagedHostStateSnapshot,
        ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
        machine_id: &MachineId,
        scenario: HostFirmwareScenario,
    ) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
        // Treat Ready (but flagged to do updates) the same as HostReprovisionState/CheckingFirmware
        let original_state = &state.managed_state.clone();
        let (mut host_reprovision_state, retry_count) = match &state.managed_state {
            ManagedHostState::HostReprovision {
                reprovision_state,
                retry_count,
            } => (reprovision_state, *retry_count),
            ManagedHostState::Ready => (
                &HostReprovisionState::CheckingFirmwareV2 {
                    firmware_type: None,
                    firmware_number: None,
                },
                0,
            ),
            ManagedHostState::Assigned { instance_state } => match &instance_state {
                InstanceState::HostReprovision { reprovision_state } => (reprovision_state, 0),
                InstanceState::Ready => (
                    &HostReprovisionState::CheckingFirmwareV2 {
                        firmware_type: None,
                        firmware_number: None,
                    },
                    0,
                ),
                _ => {
                    return Err(StateHandlerError::InvalidState(format!(
                        "Invalid state for calling handle_host_reprovision {:?}",
                        state.managed_state
                    )));
                }
            },
            _ => {
                return Err(StateHandlerError::InvalidState(format!(
                    "Invalid state for calling handle_host_reprovision {:?}",
                    state.managed_state
                )));
            }
        };

        if state
            .host_snapshot
            .host_reprovision_requested
            .as_ref()
            .is_some_and(|host_reprovision_requested| {
                host_reprovision_requested.request_reset.unwrap_or(false)
            })
        {
            tracing::info!(%machine_id, "Host firmware upgrade reset requested, returning to CheckingFirmwareRepeat");
            host_reprovision_state = &HostReprovisionState::CheckingFirmwareRepeatV2 {
                firmware_type: None,
                firmware_number: None,
            };
            state.managed_state = ManagedHostState::HostReprovision {
                reprovision_state: HostReprovisionState::CheckingFirmwareRepeatV2 {
                    firmware_type: None,
                    firmware_number: None,
                },
                retry_count: 0,
            };
            ctx.pending_db_writes.push(
                super::super::write_ops::MachineWriteOp::ResetHostReprovisioningRequest {
                    machine_id: *machine_id,
                    clear_reset: true,
                },
            );
        }

        match host_reprovision_state {
            HostReprovisionState::CheckingFirmware => {
                self.host_checking_fw(
                    &HostReprovisionState::CheckingFirmwareV2 {
                        firmware_type: None,
                        firmware_number: None,
                    },
                    state,
                    ctx,
                    original_state,
                    scenario,
                    false,
                )
                .await
            }
            HostReprovisionState::CheckingFirmwareRepeat => {
                self.host_checking_fw(
                    &HostReprovisionState::CheckingFirmwareRepeatV2 {
                        firmware_type: None,
                        firmware_number: None,
                    },
                    state,
                    ctx,
                    original_state,
                    scenario,
                    false,
                )
                .await
            }
            details @ HostReprovisionState::CheckingFirmwareV2 { .. } => {
                self.host_checking_fw(details, state, ctx, original_state, scenario, false)
                    .await
            }
            details @ HostReprovisionState::CheckingFirmwareRepeatV2 { .. } => {
                self.host_checking_fw(details, state, ctx, original_state, scenario, true)
                    .await
            }
            HostReprovisionState::WaitingForManualUpgrade { .. } => {
                self.waiting_for_manual_upgrade(state, scenario)
            }
            HostReprovisionState::WaitingForScript { .. } => {
                self.waiting_for_script(state, scenario)
            }
            HostReprovisionState::InitialReset { phase, last_time } => {
                self.pre_update_resets(
                    state,
                    ctx.services,
                    scenario,
                    Some(phase.clone()),
                    &Some(*last_time),
                )
                .await
            }
            details @ HostReprovisionState::WaitingForUpload { .. } => {
                self.waiting_for_upload(details, state, scenario, ctx).await
            }
            details @ HostReprovisionState::WaitingForFirmwareUpgrade { .. } => {
                self.host_waiting_fw(details, state, ctx, machine_id, scenario)
                    .await
            }
            details @ HostReprovisionState::ResetForNewFirmware { .. } => {
                self.host_reset_for_new_firmware(state, ctx, machine_id, details, scenario)
                    .await
            }
            details @ HostReprovisionState::NewFirmwareReportedWait { .. } => {
                self.host_new_firmware_reported_wait(state, ctx, details, machine_id, scenario)
                    .await
            }
            HostReprovisionState::FailedFirmwareUpgrade { report_time, .. } => {
                let can_retry = retry_count < MAX_FIRMWARE_UPGRADE_RETRIES;
                let waited_enough = Utc::now()
                    .signed_duration_since(report_time.unwrap_or(Utc::now()))
                    >= ctx
                        .services
                        .site_config
                        .firmware_global
                        .host_firmware_upgrade_retry_interval;
                let should_retry = can_retry && waited_enough;

                if should_retry {
                    tracing::info!("Retrying firmware upgrade on {}", state.host_snapshot.id);

                    let reprovision_state = HostReprovisionState::CheckingFirmwareV2 {
                        firmware_type: None,
                        firmware_number: None,
                    };
                    Ok(StateHandlerOutcome::transition(
                        scenario.actual_new_state(reprovision_state, retry_count + 1),
                    ))
                } else {
                    // doesn't make sense to retry anymore, remain in this failure state
                    Ok(StateHandlerOutcome::do_nothing())
                }
            }
        }
    }

    pub(super) fn is_auto_approved(&self) -> bool {
        let Some(ref period) = self.instance_autoreboot_period else {
            return false;
        };
        let start = period.start;
        let end = period.end;

        let now = chrono::Utc::now();

        now > start && now < end
    }
}

#[derive(Debug, Default)]
pub(super) struct UpdateScriptManager {
    active: Mutex<HashMap<String, Option<bool>>>,
}

impl UpdateScriptManager {
    fn started(&self, id: String) {
        let mut hashmap = self.active.lock().expect("lock poisoned");
        hashmap.insert(id, None);
    }

    fn completed(&self, id: String, success: bool) {
        let mut hashmap = self.active.lock().expect("lock poisoned");
        hashmap.insert(id, Some(success));
    }

    fn clear(&self, id: &String) {
        let mut hashmap = self.active.lock().expect("lock poisoned");
        hashmap.remove(id);
    }

    fn state(&self, id: &String) -> Option<bool> {
        let hashmap = self.active.lock().expect("lock poisoned");
        *hashmap.get(id).unwrap_or(&None)
    }
}

#[derive(Clone, Default, Debug)]
pub(super) struct AsyncFirmwareUploader {
    active_uploads: Arc<Mutex<HashMap<String, Option<UploadResult>>>>,
}

impl AsyncFirmwareUploader {
    fn start_upload(
        &self,
        id: String,
        redfish_client: Box<dyn Redfish>,
        filename: std::path::PathBuf,
        redfish_component_type: libredfish::model::update_service::ComponentType,
        address: String,
    ) {
        if self.upload_status(&id).is_some() {
            // This situation can happen during an upgrade (typically a config upgrade) where the new instance of carbide-api starts an upgrade,
            // the old one sees that it's not the uploader and returns us to Checking, then the new one is following this path.  As we would be
            // trying to return to the exact same state that we generated before and the upload is already in progress, all we need to do here is
            // return.  It's possible that we may fluctuate the state a few times, but once the old instance dies we will be fine.
            //
            // In the odd situation where the old one was doing the upload, a similar thing will happen, but when the old one dies it will kill
            // the upload and the restart is the correct thing to do.
            //
            // Log it so we can see what's going on in case there's problems.
            tracing::info!(
                "Uploading conflict for {id} {address}; our upload should still be in progress."
            );
            return;
        }
        // We set a None value to indicate that we know about this.  If we restart and we're in the next state but it's not set, we'll not find anything and know that the connection was reset.
        self.active_uploads
            .lock()
            .expect("lock poisoned")
            .insert(id.clone(), None);

        let active_uploads = self.active_uploads.clone();
        tokio::spawn(async move {
            match redfish_client
                .update_firmware_multipart(
                    filename.as_path(),
                    true,
                    std::time::Duration::from_secs(3600),
                    redfish_component_type,
                )
                .await
            {
                Ok(task_id) => {
                    let mut hashmap = active_uploads.lock().expect("lock poisoned");
                    hashmap.insert(id, Some(UploadResult::Success { task_id }));
                }
                Err(e) => {
                    tracing::warn!("Failed uploading firmware to {id} {address}: {e}");
                    let mut hashmap = active_uploads.lock().expect("lock poisoned");
                    hashmap.insert(id, Some(UploadResult::Failure));
                }
            };
        });
    }
    fn upload_status(&self, id: &String) -> Option<Option<UploadResult>> {
        let hashmap = self.active_uploads.lock().expect("lock poisoned");
        hashmap.get(id).cloned()
    }
    fn finish_upload(&self, id: &String) {
        let mut hashmap = self.active_uploads.lock().expect("lock poisoned");
        hashmap.remove(id);
    }
}
