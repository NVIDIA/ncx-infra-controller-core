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

//! Shared utility functions for the machine state handler

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::net::IpAddr;

use carbide_uuid::machine::MachineId;
use chrono::{DateTime, Duration, Utc};
use config_version::ConfigVersion;
use eyre::eyre;
use forge_secrets::credentials::{BmcCredentialType, CredentialKey};
use itertools::Itertools;
use libredfish::model::task::TaskState;
use libredfish::model::update_service::TransferProtocolType;
use libredfish::{Redfish, RedfishError, SystemPowerControl};
use model::DpuModel;
use model::firmware::{Firmware, FirmwareComponentType, FirmwareEntry};
use model::machine::{
    FailureCause, FailureDetails, InstallDpuOsState, Machine, MachineLastRebootRequested,
    MachineLastRebootRequestedMode, ManagedHostState, ManagedHostStateSnapshot, SetBootOrderInfo,
    SetBootOrderState,
};
use model::site_explorer::ExploredEndpoint;
use version_compare::Cmp;

use super::helpers::NextState;
use crate::cfg::file::{CarbideConfig, FirmwareConfig, PowerManagerOptions};
use crate::redfish::host_power_control_with_location;
use crate::state_controller::common_services::CommonStateHandlerServices;
use crate::state_controller::db_write_batch::DbWriteBatch;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::write_ops::MachineWriteOp;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub fn identify_dpu(dpu_snapshot: &Machine) -> DpuModel {
    let model = dpu_snapshot
        .hardware_info
        .as_ref()
        .and_then(|hi| {
            hi.dpu_info
                .as_ref()
                .map(|di| di.part_description.to_string())
        })
        .unwrap_or("".to_string());
    model.into()
}

// We can't use http::StatusCode because libredfish has a newer version
pub(super) const NOT_FOUND: u16 = 404;

#[cfg(not(test))]
pub const MAX_FIRMWARE_UPGRADE_RETRIES: u32 = 5;

#[cfg(test)]
pub const MAX_FIRMWARE_UPGRADE_RETRIES: u32 = 2; // Faster for tests

/// Reachability params to check if DPU is up or not.
#[derive(Copy, Clone, Debug)]
pub struct ReachabilityParams {
    pub dpu_wait_time: chrono::Duration,
    pub power_down_wait: chrono::Duration,
    pub failure_retry_time: chrono::Duration,
    pub scout_reporting_timeout: chrono::Duration,
    pub uefi_boot_wait: chrono::Duration,
}

/// Parameters used by the HostStateMachineHandler.
#[derive(Clone, Debug)]
pub struct HostHandlerParams {
    pub attestation_enabled: bool,
    pub reachability_params: ReachabilityParams,
    pub machine_validation_config: crate::cfg::file::MachineValidationConfig,
    pub bom_validation: crate::cfg::file::BomValidationConfig,
}

/// Parameters used by the Power config.
#[derive(Clone, Debug)]
pub struct PowerOptionConfig {
    pub enabled: bool,
    pub next_try_duration_on_success: chrono::TimeDelta,
    pub next_try_duration_on_failure: chrono::TimeDelta,
    pub wait_duration_until_host_reboot: chrono::TimeDelta,
}

impl From<PowerManagerOptions> for PowerOptionConfig {
    fn from(options: PowerManagerOptions) -> Self {
        Self {
            enabled: options.enabled,
            next_try_duration_on_success: options.next_try_duration_on_success,
            next_try_duration_on_failure: options.next_try_duration_on_failure,
            wait_duration_until_host_reboot: options.wait_duration_until_host_reboot,
        }
    }
}

#[derive(Clone)]
pub(super) struct FullFirmwareInfo<'a> {
    pub(super) model: &'a str,
    pub(super) to_install: &'a FirmwareEntry,
    pub(super) component_type: &'a FirmwareComponentType,
    pub(super) firmware_number: &'a u32,
}

/// need_host_fw_upgrade determines if the given endpoint needs a firmware upgrade based on the description in fw_info, and if so returns the FirmwareEntry matching the desired upgrade.
pub(super) fn need_host_fw_upgrade(
    endpoint: &ExploredEndpoint,
    fw_info: &Firmware,
    firmware_type: FirmwareComponentType,
) -> Option<FirmwareEntry> {
    // Determining if we've disabled upgrades for this host is determined in machine_update_manager, not here; if it was disabled, nothing kicks it out of Ready.

    // First, find the current version.
    let Some(current_version) = endpoint.report.versions.get(&firmware_type) else {
        // Not listed, so we couldn't do an upgrade
        return None;
    };

    // Now find the desired version, if it's not the version that is currently installed
    fw_info
        .components
        .get(&firmware_type)?
        .known_firmware
        .iter()
        .find(|x| x.default && x.version != *current_version)
        .cloned()
}

/// This function checks if reprovisioning is requested of a given DPU or not.
pub(super) fn dpu_reprovisioning_needed(dpu_snapshots: &[Machine]) -> bool {
    dpu_snapshots
        .iter()
        .any(|x| x.reprovision_requested.is_some())
}

pub(super) async fn handle_restart_verification(
    mh_snapshot: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<Option<StateHandlerOutcome<ManagedHostState>>, StateHandlerError> {
    const MAX_VERIFICATION_ATTEMPTS: i32 = 2;

    // Check host first
    if let Some(last_reboot) = &mh_snapshot.host_snapshot.last_reboot_requested
        && last_reboot.restart_verified == Some(false)
    {
        let verification_attempts = last_reboot.verification_attempts.unwrap_or(0);

        let host_redfish_client = match ctx
            .services
            .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
            .await
        {
            Ok(client) => client,
            Err(err) => {
                tracing::warn!(
                    "Failed to create Redfish client for host {} during force-restart verification: {}",
                    mh_snapshot.host_snapshot.id,
                    err
                );
                ctx.pending_db_writes
                    .push(MachineWriteOp::UpdateRestartVerificationStatus {
                        machine_id: mh_snapshot.host_snapshot.id,
                        current_reboot: *last_reboot,
                        verified: None,
                        attempts: 0,
                    });
                return Ok(None); // Skip verification, continue with state transition
            }
        };

        let restart_found = match check_restart_in_logs(
            host_redfish_client.as_ref(),
            last_reboot.time,
        )
        .await
        {
            Ok(found) => found,
            Err(err) => {
                tracing::warn!(
                    "Failed to fetch BMC logs for host {} during force-restart verification: {}",
                    mh_snapshot.host_snapshot.id,
                    err
                );
                ctx.pending_db_writes
                    .push(MachineWriteOp::UpdateRestartVerificationStatus {
                        machine_id: mh_snapshot.host_snapshot.id,
                        current_reboot: *last_reboot,
                        verified: None,
                        attempts: 0,
                    });
                return Ok(None); // Skip verification, continue with state transition
            }
        };

        if restart_found {
            ctx.pending_db_writes
                .push(MachineWriteOp::UpdateRestartVerificationStatus {
                    machine_id: mh_snapshot.host_snapshot.id,
                    current_reboot: *last_reboot,
                    verified: Some(true),
                    attempts: 0,
                });
            tracing::info!("Restart verified for host {}", mh_snapshot.host_snapshot.id);
            return Ok(None);
        }

        if verification_attempts >= MAX_VERIFICATION_ATTEMPTS {
            host_redfish_client
                .power(SystemPowerControl::ForceRestart)
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "restart host",
                    error: e,
                })?;

            ctx.pending_db_writes
                .push(MachineWriteOp::UpdateRestartVerificationStatus {
                    machine_id: mh_snapshot.host_snapshot.id,
                    current_reboot: *last_reboot,
                    verified: None,
                    attempts: 0,
                });

            tracing::info!(
                "Issued force-restart for host {} after {} failed verifications",
                mh_snapshot.host_snapshot.id,
                verification_attempts
            );
            return Ok(None);
        }

        ctx.pending_db_writes
            .push(MachineWriteOp::UpdateRestartVerificationStatus {
                machine_id: mh_snapshot.host_snapshot.id,
                current_reboot: *last_reboot,
                verified: Some(false),
                attempts: verification_attempts + 1,
            });

        return Ok(Some(StateHandlerOutcome::wait(format!(
            "Waiting for {} force-restart verification - attempt {}/{}",
            mh_snapshot.host_snapshot.id,
            verification_attempts + 1,
            MAX_VERIFICATION_ATTEMPTS
        ))));
    }

    // Check DPUs
    let mut pending_message = Vec::new();

    for dpu in &mh_snapshot.dpu_snapshots {
        if let Some(last_reboot) = dpu.last_reboot_requested
            && last_reboot.restart_verified == Some(false)
        {
            let verification_attempts = last_reboot.verification_attempts.unwrap_or(0);

            let dpu_redfish_client = match ctx
                .services
                .create_redfish_client_from_machine(dpu)
                .await
            {
                Ok(client) => client,
                Err(err) => {
                    tracing::warn!(
                        "Failed to create Redfish client for DPU {} during force-restart verification: {}",
                        dpu.id,
                        err
                    );
                    ctx.pending_db_writes
                        .push(MachineWriteOp::UpdateRestartVerificationStatus {
                            machine_id: dpu.id,
                            current_reboot: last_reboot,
                            verified: None,
                            attempts: 0,
                        });
                    continue; // Skip verification, continue with state transition
                }
            };

            let restart_found = match check_restart_in_logs(
                dpu_redfish_client.as_ref(),
                last_reboot.time,
            )
            .await
            {
                Ok(found) => found,
                Err(err) => {
                    tracing::warn!(
                        "Failed to fetch BMC logs for DPU {} during force-restart verification: {}",
                        dpu.id,
                        err
                    );

                    ctx.pending_db_writes
                        .push(MachineWriteOp::UpdateRestartVerificationStatus {
                            machine_id: dpu.id,
                            current_reboot: last_reboot,
                            verified: None,
                            attempts: 0,
                        });

                    continue; // Skip verification, continue with state transition
                }
            };

            if restart_found {
                ctx.pending_db_writes
                    .push(MachineWriteOp::UpdateRestartVerificationStatus {
                        machine_id: dpu.id,
                        current_reboot: last_reboot,
                        verified: Some(true),
                        attempts: 0,
                    });
                tracing::info!("Restart verified for DPU {}", dpu.id);
            } else if verification_attempts >= MAX_VERIFICATION_ATTEMPTS {
                dpu_redfish_client
                    .power(SystemPowerControl::ForceRestart)
                    .await
                    .map_err(|e| StateHandlerError::RedfishError {
                        operation: "reboot dpu",
                        error: e,
                    })?;

                ctx.pending_db_writes
                    .push(MachineWriteOp::UpdateRestartVerificationStatus {
                        machine_id: dpu.id,
                        current_reboot: last_reboot,
                        verified: None,
                        attempts: 0,
                    });

                tracing::info!(
                    "Issued force-restart for DPU {} after {} failed verifications",
                    dpu.id,
                    verification_attempts
                );
            } else {
                ctx.pending_db_writes
                    .push(MachineWriteOp::UpdateRestartVerificationStatus {
                        machine_id: dpu.id,
                        current_reboot: last_reboot,
                        verified: Some(false),
                        attempts: verification_attempts + 1,
                    });

                pending_message.push(format!(
                    "DPU {} force-restart verification - attempt {}/{}",
                    dpu.id,
                    verification_attempts + 1,
                    MAX_VERIFICATION_ATTEMPTS
                ));
            }
        }
    }

    if !pending_message.is_empty() {
        Ok(Some(StateHandlerOutcome::wait(pending_message.join(", "))))
    } else {
        Ok(None)
    }
}

pub async fn check_restart_in_logs(
    redfish_client: &dyn Redfish,
    restart_time: DateTime<Utc>,
) -> Result<bool, RedfishError> {
    lazy_static::lazy_static! {
        // Vendor specific messages
        static ref SPECIFIC_RESET_KEYWORDS: HashSet<&'static str> = HashSet::from([
            "Server reset.",                                       // HPE
            "Server power restored.",                              // HPE
            "The server is restarted by chassis control command.", // Lenovo
            "DPU Warm Reset",                                      // Bluefield
            "BMC IP Address Deleted",                              // Bluefield
        ]);

        // Generic reset keywords
        static ref GENERIC_RESET_KEYWORDS: Vec<&'static str> =
            vec!["reset", "reboot", "restart", "power", "start"];
    }

    let logs = redfish_client.get_bmc_event_log(Some(restart_time)).await?;

    for log in &logs {
        tracing::debug!("BMC log message: {}", log.message);
    }

    let restart_found = logs.iter().any(|log| {
        // First check exact matches
        if SPECIFIC_RESET_KEYWORDS.contains(log.message.as_str()) {
            return true;
        }
        // Then generic keywords
        let lowercase_message = log.message.to_lowercase();
        GENERIC_RESET_KEYWORDS
            .iter()
            .any(|keyword| lowercase_message.contains(keyword))
    });

    Ok(restart_found)
}

// Function to wait for some time in state machine.
pub(super) fn wait(basetime: &DateTime<Utc>, wait_time: Duration) -> bool {
    let expected_time = *basetime + wait_time;
    let current_time = Utc::now();

    current_time < expected_time
}

pub(super) fn is_dpu_up(state: &ManagedHostStateSnapshot, dpu_snapshot: &Machine) -> bool {
    let observation_time = dpu_snapshot
        .network_status_observation
        .as_ref()
        .map(|o| o.observed_at)
        .unwrap_or(DateTime::<Utc>::MIN_UTC);
    let state_change_time = state.host_snapshot.state.version.timestamp();

    if observation_time < state_change_time {
        return false;
    }

    true
}

/// are_dpus_up_trigger_reboot_if_needed returns true if the dpu_agent indicates that the DPU has rebooted and is healthy.
/// otherwise returns false. triggers a reboot in case the DPU is down/bricked.
pub(super) async fn are_dpus_up_trigger_reboot_if_needed(
    state: &ManagedHostStateSnapshot,
    reachability_params: &ReachabilityParams,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> bool {
    for dpu_snapshot in &state.dpu_snapshots {
        if !is_dpu_up(state, dpu_snapshot) {
            match trigger_reboot_if_needed(dpu_snapshot, state, None, reachability_params, ctx)
                .await
            {
                Ok(_) => {}
                Err(e) => tracing::warn!("could not reboot dpu {}: {e}", dpu_snapshot.id),
            }
            return false;
        }
    }

    true
}

// Returns true if update_manager flagged this managed host as needing its firmware examined
pub(super) fn host_reprovisioning_requested(state: &ManagedHostStateSnapshot) -> bool {
    state.host_snapshot.host_reprovision_requested.is_some()
}

/// This function waits for DPU to finish discovery and reboots it.
pub async fn try_wait_for_dpu_discovery(
    state: &ManagedHostStateSnapshot,
    reachability_params: &ReachabilityParams,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    is_reprovision_case: bool,
    current_dpu_machine_id: &MachineId,
) -> Result<Option<MachineId>, StateHandlerError> {
    // We are waiting for the `DiscoveryCompleted` RPC call to update the
    // `last_discovery_time` timestamp.
    // This indicates that all forge-scout actions have succeeded.
    for dpu_snapshot in &state.dpu_snapshots {
        if is_reprovision_case && dpu_snapshot.reprovision_requested.is_none() {
            // This is reprovision handling and this DPU is not under reprovisioning.
            continue;
        }
        if !discovered_after_state_transition(
            dpu_snapshot.state.version,
            dpu_snapshot.last_discovery_time,
        ) {
            // Reboot only the DPU for which the handler loop is called.
            if current_dpu_machine_id == &dpu_snapshot.id {
                let _status =
                    trigger_reboot_if_needed(dpu_snapshot, state, None, reachability_params, ctx)
                        .await?;
            }
            // TODO propagate the status.status message to a StateHandlerOutcome::Wait
            return Ok(Some(dpu_snapshot.id));
        }
    }

    Ok(None)
}

/// Returns Option<StateHandlerOutcome>:
///     If Some(_) means at least one fw component is not updated.
///     If None: All fw components are updated.
pub(super) async fn check_fw_component_version(
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    dpu_snapshot: &Machine,
    hardware_models: &FirmwareConfig,
) -> Result<Option<StateHandlerOutcome<ManagedHostState>>, StateHandlerError> {
    let redfish_client = ctx
        .services
        .create_redfish_client_from_machine(dpu_snapshot)
        .await?;

    let redfish_component_name_map = HashMap::from([
        // Note: DPU uses different name for BMC Firmware as
        // BF2: 6d53cf4d_BMC_Firmware
        // BF3: BMC_Firmware
        (FirmwareComponentType::Nic, "DPU_NIC"),
        (FirmwareComponentType::Bmc, "BMC_Firmware"),
        (FirmwareComponentType::Uefi, "DPU_UEFI"),
        (FirmwareComponentType::Cec, "Bluefield_FW_ERoT"),
    ]);
    let inventories = redfish_client
        .get_software_inventories()
        .await
        .map_err(|e| StateHandlerError::RedfishError {
            operation: "get_software_inventories",
            error: e,
        })?;

    for component in [
        FirmwareComponentType::Bmc,
        FirmwareComponentType::Cec,
        FirmwareComponentType::Nic,
    ] {
        let component_name = redfish_component_name_map.get(&component).unwrap();
        let inventory_id = inventories
            .iter()
            .find(|i| i.contains(component_name))
            .ok_or(StateHandlerError::FirmwareUpdateError(eyre!(
                "No inventory found that matches redfish component name: {component_name}; inventory list: {inventories:#?}",
            )))?;

        let inventory = match redfish_client.get_firmware(inventory_id).await {
            Ok(inventory) => inventory,
            Err(e) => {
                tracing::error!(machine_id=%dpu_snapshot.id, "redfish command get_firmware error {}", e.to_string());
                return Err(StateHandlerError::RedfishError {
                    operation: "get_firmware",
                    error: e,
                });
            }
        };

        if inventory.version.is_none() {
            let msg = format!("Unknown {component_name:?} version");
            tracing::error!(machine_id=%dpu_snapshot.id, msg);
            return Err(StateHandlerError::FirmwareUpdateError(eyre!(msg)));
        };

        let cur_version = inventory
            .version
            .unwrap_or("Unknown current installed BMC FW version".to_string());

        let model = identify_dpu(dpu_snapshot);

        let expected_version = hardware_models
            .find(bmc_vendor::BMCVendor::Nvidia, &model.to_string())
            .and_then(|fw| fw.components.get(&component).cloned())
            .and_then(|fw_component| {
                fw_component
                    .known_firmware
                    .iter()
                    .filter(|fw_entry| !fw_entry.preingestion_exclusive_config)
                    .next_back()
                    .cloned()
            })
            .map(|f| f.version)
            .unwrap_or("Unknown current configured BMC FW version".to_string());

        if cur_version != expected_version {
            // CEC_MIN_RESET_VERSION="00.02.0180.0000"
            if component == FirmwareComponentType::Cec
                && version_compare::compare_to(&cur_version, "00.02.0180.0000", Cmp::Lt)
                    .is_ok_and(|x| x)
            {
                // For this case need to run host power cycle
                tracing::info!(
                    machine_id=%dpu_snapshot.id,
                    "Need to launch host power cycle to update CEC FW from {} to {}",
                    cur_version,
                    expected_version
                );
                return Ok(None);
            }

            tracing::warn!(
                machine_id=%dpu_snapshot.id,
                "{:#?} FW didn't update succesfully. Expected version: {}, Current version: {}",
                component,
                expected_version,
                cur_version,
            );

            // Don't return Error. In case of the error, reboot time won't be updated in db.
            // This will cause continuous reboot of machine after first failure_retry_time is
            // passed.
            return Ok(Some(StateHandlerOutcome::wait(format!(
                "{:#?} FW didn't update succesfully. Expected version: {}, Current version: {}",
                component, expected_version, cur_version,
            ))));
        }

        tracing::info!(
            machine_id=%dpu_snapshot.id,
            "{:#?} FW updated succesfully to {}",
            component,
            expected_version,
        );

        // BMC FW version need to update in machine_topology->bmc_info
        if component == FirmwareComponentType::Bmc
            && dpu_snapshot
                .bmc_info
                .clone()
                .firmware_version
                .is_some_and(|v| v != cur_version)
            && let Some(dpu_bmc_ip) = dpu_snapshot.bmc_addr().map(|a| a.ip())
        {
            let bios_version: String = redfish_client
                .get_firmware("DPU_UEFI")
                .await
                .inspect_err(|e| {
                    tracing::error!("redfish command get_firmware error {}", e.to_string());
                    tracing::error!(machine_id=%dpu_snapshot.id, "redfish command get_firmware error {}", e.to_string());
                })
                .ok()
                .and_then(|uefi| uefi.version)
                .unwrap_or_else(|| {
                    dpu_snapshot
                        .hardware_info
                        .as_ref()
                        .and_then(|h| h.dmi_data.as_ref())
                        .map(|d| d.bios_version.clone())
                        .unwrap_or_default()
                });

            ctx.pending_db_writes.push(
                // This is safe to defer to pending_db_writes because this is a no-op if for some
                // reason dpu_bmc_ip is not found.
                MachineWriteOp::UpdateFirmwareVersionByBmcAddress {
                    bmc_address: dpu_bmc_ip,
                    bmc_version: cur_version,
                    bios_version,
                },
            );
        }
    }

    // All good.
    Ok(None)
}

pub(super) fn set_managed_host_topology_update_needed(
    pending_db_writes: &mut DbWriteBatch,
    host_snapshot: &Machine,
    dpus: &[&Machine],
) {
    //Update it for host and DPU both.
    for dpu_snapshot in dpus {
        pending_db_writes.push(MachineWriteOp::SetTopologyUpdateNeeded {
            machine_id: dpu_snapshot.id,
            value: true,
        });
    }

    pending_db_writes.push(MachineWriteOp::SetTopologyUpdateNeeded {
        machine_id: host_snapshot.id,
        value: true,
    });
}

/// This function returns failure cause for both host and dpu.
pub(super) fn get_failed_state(
    state: &ManagedHostStateSnapshot,
) -> Option<(MachineId, FailureDetails)> {
    // Return updated state only for errors which should cause machine to move into failed
    // state.
    if state.host_snapshot.failure_details.cause != FailureCause::NoError {
        return Some((
            state.host_snapshot.id,
            state.host_snapshot.failure_details.clone(),
        ));
    } else {
        for dpu_snapshot in &state.dpu_snapshots {
            // In case of the DPU, use first failed DPU and recover it before moving forward.
            if dpu_snapshot.failure_details.cause != FailureCause::NoError {
                return Some((dpu_snapshot.id, dpu_snapshot.failure_details.clone()));
            }
        }
    }

    None
}

pub(super) fn get_reboot_cycle(
    next_potential_reboot_time: DateTime<Utc>,
    entered_state_at: DateTime<Utc>,
    wait_period: Duration,
) -> Result<i64, StateHandlerError> {
    if next_potential_reboot_time <= entered_state_at {
        return Err(StateHandlerError::GenericError(eyre::eyre!(
            "Poorly configured paramters: next_potential_reboot_time: {}, entered_state_at: {}, wait_period: {}",
            next_potential_reboot_time,
            entered_state_at,
            wait_period.num_minutes()
        )));
    }

    let cycle = next_potential_reboot_time - entered_state_at;

    // Although trigger_reboot_if_needed makes sure to not send wait_period as 0, but still if some other
    // function calls get_reboot_cycle, this function must not panic, so setting it min 1 minute
    // here as well.
    Ok(cycle.num_minutes() / wait_period.num_minutes().max(1))
}

#[derive(Debug)]
pub struct RebootStatus {
    pub increase_retry_count: bool, // the vague previous return value
    pub status: String,             // what we did or are waiting for
}

/// Outcome of configure_host_bios function.
pub(super) enum BiosConfigOutcome {
    Done,
    WaitingForReboot(String),
}

/// Outcome of set_host_boot_order function.
pub(super) enum SetBootOrderOutcome {
    Continue(SetBootOrderInfo),
    Done,
    WaitingForReboot(String),
}

/// In case machine does not come up until a specified duration, this function tries to reboot
/// it again. The reboot continues till 6 hours only. After that this function gives up.
/// WARNING:
/// If using this function in handler, never return Error, return wait/donothing.
/// In case a error is returned, last_reboot_requested won't be updated in db by state handler.
/// This will cause continuous reboot of machine after first failure_retry_time is
/// passed.
#[track_caller]
pub fn trigger_reboot_if_needed(
    target: &Machine,
    state: &ManagedHostStateSnapshot,
    retry_count: Option<i64>,
    reachability_params: &ReachabilityParams,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> impl Future<Output = Result<RebootStatus, StateHandlerError>> {
    let trigger_location = std::panic::Location::caller();
    trigger_reboot_if_needed_with_location(
        target,
        state,
        retry_count,
        reachability_params,
        ctx,
        trigger_location,
    )
}

pub async fn trigger_reboot_if_needed_with_location(
    target: &Machine,
    state: &ManagedHostStateSnapshot,
    retry_count: Option<i64>,
    reachability_params: &ReachabilityParams,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    trigger_location: &std::panic::Location<'_>,
) -> Result<RebootStatus, StateHandlerError> {
    let host = &state.host_snapshot;
    // Its highly unlikely that the host has never been rebooted (and the last_reboot_reqeusted
    // field shouldn't get cleared), but default it if its not set
    let last_reboot_requested = match &target.last_reboot_requested {
        None => &MachineLastRebootRequested {
            time: host.state.version.timestamp(),
            mode: MachineLastRebootRequestedMode::Reboot,
            ..MachineLastRebootRequested::default()
        },
        Some(req) => req,
    };

    if let MachineLastRebootRequestedMode::PowerOff = last_reboot_requested.mode {
        // PowerOn the host.
        tracing::info!(
            "Machine {} is in power-off state. Turning on for host: {}",
            target.id,
            host.id,
        );

        if wait(
            &last_reboot_requested.time,
            reachability_params.power_down_wait,
        ) {
            return Ok(RebootStatus {
                increase_retry_count: false,
                status: format!(
                    "Waiting for host to power off. Next check at {}",
                    last_reboot_requested.time + reachability_params.power_down_wait
                ),
            });
        }

        let redfish_client = ctx
            .services
            .create_redfish_client_from_machine(host)
            .await?;

        let power_state = host_power_state(redfish_client.as_ref()).await?;

        // If power-off done, power-on now.
        // If host is not powered-off yet, try again.
        let action = if power_state == libredfish::PowerState::Off {
            SystemPowerControl::On
        } else {
            tracing::error!(
                "Machine {} is still not power-off state. Turning off again for host: {}",
                target.id,
                host.id,
            );
            SystemPowerControl::ForceOff
        };

        tracing::trace!(machine_id=%target.id, "Redfish setting host power state to {action}");
        handler_host_power_control_with_location(state, ctx, action, trigger_location).await?;
        return Ok(RebootStatus {
            increase_retry_count: false,
            status: format!("Set power state to {action} using Redfish API"),
        });
    }

    // Check if reboot is prevented by health override.
    if state.aggregate_health.is_reboot_blocked_in_state_machine() {
        tracing::info!(
            "Not trying to reboot {} since health override is set to prevent reboot.",
            target.id,
        );
        return Ok(RebootStatus {
            increase_retry_count: false,
            status: format!(
                "Not trying to reboot {} since health override is set to prevent reboot.",
                target.id
            ),
        });
    }

    let wait_period = reachability_params
        .failure_retry_time
        .max(Duration::minutes(1));

    let current_time = Utc::now();
    let entered_state_at = target.state.version.timestamp();
    let next_potential_reboot_time: DateTime<Utc> =
        if last_reboot_requested.time + wait_period > entered_state_at {
            last_reboot_requested.time + wait_period
        } else {
            // Handles this case:
            // T0: State A
            //      DPU was hung--Reboot DPU
            //      DPU last requested reboot requested time: T0
            // T1 (T0 + 1 hour): State B
            //      DPU was hung; DPU wait period is 45 mins
            //      If we only calculate the next reboot time from the last requested reboot time
            //      the DPU's next potential reboot time = T0 + 45 < T1
            // Our logic to detect the reboot cycle will return an error here,
            // because the next reboot time is before the time the DPU entered State B.
            // Update the DPU's next reboot time to be 5 minutes after it entered State B to handle
            // this edge case.
            entered_state_at + Duration::minutes(5)
        };

    let time_elapsed_since_state_change = (current_time - entered_state_at).num_minutes();
    // Let's stop at 15 cycles of reboot.
    let max_retry_duration = Duration::minutes(wait_period.num_minutes() * 15);

    let should_try = if let Some(retry_count) = retry_count {
        retry_count < 15
    } else {
        entered_state_at + max_retry_duration > current_time
    };

    // We can try reboot only upto 15 cycles from state change.
    if should_try {
        // A cycle is done but host has not responded yet. Let's try a reboot.
        if next_potential_reboot_time < current_time {
            // Find the cycle.
            // We are trying to reboot 3 times and power down/up on 4th cycle.
            let cycle = match retry_count {
                Some(x) => x,
                None => {
                    get_reboot_cycle(next_potential_reboot_time, entered_state_at, wait_period)?
                }
            };

            // Dont power down the host on the first cycle
            let power_down_host = cycle != 0 && cycle % 4 == 0;

            let status = if power_down_host {
                // PowerDown (or ACPowercycle for Lenovo)
                // DPU or host, in both cases power down is triggered from host.
                let vendor = state.host_snapshot.bmc_vendor();

                let action = if vendor.is_lenovo() {
                    SystemPowerControl::ACPowercycle
                } else {
                    SystemPowerControl::ForceOff
                };

                handler_host_power_control_with_location(state, ctx, action, trigger_location)
                    .await?;

                format!(
                    "{vendor} has not come up after {time_elapsed_since_state_change} minutes, trying {action}, cycle: {cycle}",
                )
            } else {
                // Reboot
                if target.id.machine_type().is_dpu() {
                    handler_restart_dpu(target, ctx, state.host_snapshot.dpf.used_for_ingestion)
                        .await?;
                } else {
                    if let Ok(client) = ctx.services.create_redfish_client_from_machine(host).await
                    {
                        log_host_config(client.as_ref(), state).await;
                    }

                    handler_host_power_control_with_location(
                        state,
                        ctx,
                        SystemPowerControl::ForceRestart,
                        trigger_location,
                    )
                    .await?;
                }
                format!(
                    "Has not come up after {time_elapsed_since_state_change} minutes. Rebooting again, cycle: {cycle}."
                )
            };

            tracing::info!(machine_id=%target.id,
                "triggered reboot for machine in managed-host state {}: {}",
                state.managed_state,
                status,
            );

            Ok(RebootStatus {
                increase_retry_count: true,
                status,
            })
        } else {
            Ok(RebootStatus {
                increase_retry_count: false,
                status: format!("Will attempt next reboot at {next_potential_reboot_time}"),
            })
        }
    } else {
        let h = (current_time - entered_state_at).num_hours();
        Err(StateHandlerError::ManualInterventionRequired(format!(
            "Machine has not responded after {h} hours."
        )))
    }
}

/// This function waits until target machine is up or not. It relies on scout to identify if
/// machine has come up or not after reboot.
// True if machine is rebooted after state change.
pub fn rebooted(target: &Machine) -> bool {
    target.last_reboot_time.unwrap_or_default() > target.state.version.timestamp()
}

pub fn machine_validation_completed(target: &Machine) -> bool {
    target.last_machine_validation_time.unwrap_or_default() > target.state.version.timestamp()
}
// Was machine rebooted after state change?
pub(super) fn discovered_after_state_transition(
    version: ConfigVersion,
    last_discovery_time: Option<DateTime<Utc>>,
) -> bool {
    last_discovery_time.unwrap_or_default() > version.timestamp()
}

// Was DPU reprov restart requested after state change
pub(super) fn dpu_reprovision_restart_requested_after_state_transition(
    version: ConfigVersion,
    reprov_restart_requested_at: DateTime<Utc>,
) -> bool {
    reprov_restart_requested_at > version.timestamp()
}

pub(super) fn cleanedup_after_state_transition(
    version: ConfigVersion,
    last_cleanup_time: Option<DateTime<Utc>>,
) -> bool {
    last_cleanup_time.unwrap_or_default() > version.timestamp()
}

pub(super) fn check_host_health_for_alerts(
    state: &ManagedHostStateSnapshot,
) -> Result<(), StateHandlerError> {
    // In some states, DPU alerts may be surpressed (classifications removed) in the aggregate health report.
    // Since this is not called from a state that supresses DPU alerts, this is ok here.
    match state
        .aggregate_health
        .has_classification(&health_report::HealthAlertClassification::prevent_host_state_changes())
    {
        true => Err(StateHandlerError::HealthProbeAlert),
        false => Ok(()),
    }
}

#[track_caller]
pub(super) fn handler_restart_dpu(
    machine: &Machine,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    dpf_used_for_ingestion: bool,
) -> impl Future<Output = Result<(), StateHandlerError>> {
    let trigger_location = std::panic::Location::caller();
    tracing::info!(
        dpu_machine_id = %machine.id,
        %trigger_location,
        "DPU restart triggered"
    );
    ctx.pending_db_writes
        .push(MachineWriteOp::UpdateRebootRequestedTime {
            machine_id: machine.id,
            mode: model::machine::MachineLastRebootRequestedMode::Reboot,
            time: Utc::now(),
        });
    restart_dpu(machine, ctx.services, dpf_used_for_ingestion)
}

pub async fn host_power_state(
    redfish_client: &dyn Redfish,
) -> Result<libredfish::PowerState, StateHandlerError> {
    redfish_client
        .get_power_state()
        .await
        .map_err(|e| StateHandlerError::RedfishError {
            operation: "get_power_state",
            error: e,
        })
}

pub(super) fn requires_manual_firmware_upgrade(
    state: &ManagedHostStateSnapshot,
    config: &CarbideConfig,
) -> bool {
    if !config.firmware_global.requires_manual_upgrade {
        return false;
    }

    let is_gb200 = state
        .host_snapshot
        .hardware_info
        .as_ref()
        .map(|hi| hi.is_gbx00())
        .unwrap_or(false);

    if !is_gb200 {
        return false;
    }

    state
        .host_snapshot
        .manual_firmware_upgrade_completed
        .is_none()
}

#[track_caller]
pub fn handler_host_power_control(
    managedhost_snapshot: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    action: SystemPowerControl,
) -> impl Future<Output = Result<(), StateHandlerError>> {
    let trigger_location = std::panic::Location::caller();
    handler_host_power_control_with_location(managedhost_snapshot, ctx, action, trigger_location)
}

pub async fn handler_host_power_control_with_location(
    managedhost_snapshot: &ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    action: SystemPowerControl,
    location: &std::panic::Location<'_>,
) -> Result<(), StateHandlerError> {
    let mut action = action;
    let redfish_client = ctx
        .services
        .create_redfish_client_from_machine(&managedhost_snapshot.host_snapshot)
        .await?;

    let power_state = host_power_state(redfish_client.as_ref()).await?;

    let target_power_state_reached = (power_state == libredfish::PowerState::Off
        && (action == SystemPowerControl::ForceOff
            || action == SystemPowerControl::GracefulShutdown))
        || (power_state == libredfish::PowerState::On && action == SystemPowerControl::On);

    if target_power_state_reached {
        let machine_id = &managedhost_snapshot.host_snapshot.id;
        tracing::warn!(%machine_id, %power_state, %action, "Target power state is already reached. Skipping power control action");
    } else {
        if power_state == libredfish::PowerState::Off
            && (action == SystemPowerControl::ForceRestart
                || action == SystemPowerControl::GracefulRestart)
        {
            // A host can't be restarted if it is in power-off state.
            // In this call, power on the system. State machine restart the system in next iteration.
            tracing::warn!(%power_state, %action, "Power state is Off and requested action is restart. Trying to power on the host.");
            action = SystemPowerControl::On;
        }

        let machine = &managedhost_snapshot.host_snapshot;
        let is_restart = action == SystemPowerControl::ForceRestart
            || action == SystemPowerControl::GracefulRestart;

        if is_restart && needs_ipmi_restart(machine, ctx).await? {
            do_ipmi_restart(machine, ctx, action, location).await?;
        } else {
            host_power_control_with_location(
                redfish_client.as_ref(),
                machine,
                action,
                ctx,
                location,
            )
            .await
            .map_err(|e| {
                StateHandlerError::GenericError(eyre!("handler_host_power_control failed: {}", e))
            })?;
        }
    }

    // If host is forcedOff/ACPowercycled/On, it will impact DPU also. So DPU timestamp should also be updated
    // here.
    let dpu_impacting_actions = [
        SystemPowerControl::ForceOff,
        SystemPowerControl::ACPowercycle,
        SystemPowerControl::On,
    ];
    let should_update_dpu_timestamp = dpu_impacting_actions.contains(&action);

    if should_update_dpu_timestamp {
        for dpu_snapshot in &managedhost_snapshot.dpu_snapshots {
            ctx.pending_db_writes
                .push(MachineWriteOp::UpdateRebootRequestedTime {
                    machine_id: dpu_snapshot.id,
                    mode: action.into(),
                    time: Utc::now(),
                });
        }
    }

    Ok(())
}

pub(super) async fn restart_dpu(
    machine: &Machine,
    services: &CommonStateHandlerServices,
    dpf_used_for_ingestion: bool,
) -> Result<(), StateHandlerError> {
    let dpu_redfish_client = services.create_redfish_client_from_machine(machine).await?;

    // We have seen the boot order be reset on DPUs in some edge cases (for example, after upgrading the BMC and CEC on BF3s)
    // This should take care of handling such cases. It is a no-op most of the time.
    // Skip for DPUs that get their BFB installed via redfish or DPF, they don't need to HTTP boot.
    let redfish_install = machine.bmc_info.supports_bfb_install()
        && services.site_config.dpu_config.dpu_enable_secure_boot;

    if !redfish_install && !dpf_used_for_ingestion {
        let _ = dpu_redfish_client
            .boot_once(libredfish::Boot::UefiHttp)
            .await
            .map_err(|e| {
                // We use a Dell to mock our BMC responses in the integration tests. UefiHttp boot is not implemented
                // for Dells, so this call is failing in our tests. Regardless, it is ok to not make this call blocking.
                tracing::error!(%e, "Failed to configure DPU {} to boot once", machine.id);
            });
    }

    if let Err(e) = dpu_redfish_client
        .power(SystemPowerControl::ForceRestart)
        .await
    {
        tracing::error!(%e, "Failed to reboot a DPU");
        return Err(StateHandlerError::RedfishError {
            operation: "reboot dpu",
            error: e,
        });
    }

    Ok(())
}

/// Returns true if this machine needs IPMI restart to avoid killing its DPUs.
/// Redfish restart kills the DPU on some machines
pub(super) async fn needs_ipmi_restart(
    machine: &Machine,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<bool, StateHandlerError> {
    let addr = machine
        .bmc_info
        .ip_addr()
        .map_err(StateHandlerError::GenericError)?;
    let endpoints =
        db::explored_endpoints::find_by_ips(&mut ctx.services.db_reader, vec![addr]).await?;
    let Some(ep) = endpoints.first() else {
        return Ok(false);
    };

    Ok(match ep.report.vendor {
        // Lenovo SR650 V4s kill power to DPUs on Redfish ForceRestart/GracefulRestart,
        // causing PXE boot failures. IPMI chassis reset avoids this.
        // https://github.com/NVIDIA/bare-metal-manager-core/issues/347
        Some(bmc_vendor::BMCVendor::Lenovo) => {
            let model = ep.report.model.as_deref().unwrap_or("");
            model.contains("SR650 V4")
        }
        Some(bmc_vendor::BMCVendor::Nvidia) => {
            ep.report.systems.iter().any(|s| s.id == "DGX")
                && ep.report.managers.iter().any(|m| m.id == "BMC")
        }
        _ => false,
    })
}

/// Perform an IPMI chassis power reset for the given machine
pub(super) async fn do_ipmi_restart(
    machine: &Machine,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    action: SystemPowerControl,
    trigger_location: &std::panic::Location<'_>,
) -> Result<(), StateHandlerError> {
    tracing::info!(
        machine_id = machine.id.to_string(),
        action = action.to_string(),
        trigger_location = %trigger_location,
        "IPMI Host Power Control"
    );
    ctx.pending_db_writes
        .push(MachineWriteOp::UpdateRebootRequestedTime {
            machine_id: machine.id,
            mode: action.into(),
            time: Utc::now(),
        });

    let bmc_mac = machine
        .bmc_info
        .mac
        .ok_or_else(|| StateHandlerError::MissingData {
            object_id: machine.id.to_string(),
            missing: "bmc_mac",
        })?;
    let ip: IpAddr = machine
        .bmc_info
        .ip
        .as_ref()
        .ok_or_else(|| StateHandlerError::MissingData {
            object_id: machine.id.to_string(),
            missing: "bmc_ip",
        })?
        .parse()
        .map_err(|e| {
            StateHandlerError::GenericError(eyre!(
                "parsing BMC IP address for {} failed: {}",
                machine.id,
                e
            ))
        })?;
    let credential_key = CredentialKey::BmcCredentials {
        credential_type: BmcCredentialType::BmcRoot {
            bmc_mac_address: bmc_mac,
        },
    };
    ctx.services
        .ipmi_tool
        .restart(&machine.id, ip, false, &credential_key)
        .await
        .map_err(|e| {
            StateHandlerError::GenericError(eyre!("IPMI restart failed for {}: {}", machine.id, e))
        })
}

/// find_explored_refreshed_endpoint will locate the explored endpoint for the given state.
/// It will return an error for not finding any endpoint, and Ok(None) when we're still waiting
/// on explorer to have a chance to run again.
pub async fn find_explored_refreshed_endpoint(
    state: &ManagedHostStateSnapshot,
    machine_id: &MachineId,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<Option<ExploredEndpoint>, StateHandlerError> {
    let addr: IpAddr = state
        .host_snapshot
        .bmc_info
        .ip_addr()
        .map_err(StateHandlerError::GenericError)?;

    let endpoint =
        db::explored_endpoints::find_by_ips(&mut ctx.services.db_reader, vec![addr]).await?;
    let endpoint = endpoint
        .into_iter()
        .next()
        .ok_or(StateHandlerError::GenericError(
            eyre! {"Unable to find explored_endpoint for {machine_id}"},
        ))?;

    if endpoint.waiting_for_explorer_refresh {
        // In the cases where this was called, we care about prompt updates, so poke site explorer to revisit this endpoint next time it runs
        ctx.pending_db_writes
            .push(MachineWriteOp::ReExploreIfVersionMatches {
                address: endpoint.address,
                version: endpoint.report_version,
            });
        return Ok(None);
    }
    Ok(Some(endpoint))
}

// If already reprovisioning is started, we can restart.
// Also check that this is not some old request. The restart requested time must be greater than
// last state change.
pub(super) fn can_restart_reprovision(dpu_snapshots: &[Machine], version: ConfigVersion) -> bool {
    let mut reprov_started = false;
    let mut requested_at = vec![];
    for dpu_snapshot in dpu_snapshots {
        if let Some(reprov_req) = &dpu_snapshot.reprovision_requested {
            if reprov_req.started_at.is_some() {
                reprov_started = true;
            }
            requested_at.push(reprov_req.restart_reprovision_requested_at);
        }
    }

    if !reprov_started {
        return false;
    }

    // Get the latest time of restart requested.
    requested_at.sort();

    let Some(latest_requested_at) = requested_at.last() else {
        return false;
    };

    dpu_reprovision_restart_requested_after_state_transition(version, *latest_requested_at)
}

/// Call [`Redfish::machine_setup`], but ignore any [`RedfishError::NoDpu`] if we expect there to be no DPUs.
///
/// TODO(ken): This is a temporary workaround for work-in-progress on zero-DPU support (August 2024)
/// The way we should do this going forward is to plumb the actual non-DPU MAC address we want to
/// boot from, but that information is not in scope at this time. Once it is, and we pass it to
/// machine_setup, we should no longer expect a NoDpu error and can thus call vanilla machine_setup again.
pub(super) async fn call_machine_setup_and_handle_no_dpu_error(
    redfish_client: &dyn Redfish,
    boot_interface_mac: Option<&str>,
    expected_dpu_count: usize,
    site_config: &CarbideConfig,
) -> Result<(), RedfishError> {
    let setup_result = redfish_client
        .machine_setup(
            boot_interface_mac,
            &site_config.bios_profiles,
            site_config.selected_profile,
            &HashMap::default(),
        )
        .await;
    match (
        setup_result,
        expected_dpu_count,
        site_config.site_explorer.allow_zero_dpu_hosts,
    ) {
        (Err(RedfishError::NoDpu), 0, true) => {
            tracing::info!(
                "redfish machine_setup failed due to there being no DPUs on the host. This is expected as the host has no DPUs, and we are configured to allow this."
            );
            Ok(())
        }
        (Ok(()), _, _) => Ok(()),
        (Err(e), _, _) => Err(e),
    }
}

pub(super) async fn set_boot_order_dpu_first_and_handle_no_dpu_error(
    redfish_client: &dyn Redfish,
    boot_interface_mac: &str,
    expected_dpu_count: usize,
    site_config: &CarbideConfig,
) -> Result<Option<String>, RedfishError> {
    let setup_result = redfish_client
        .set_boot_order_dpu_first(boot_interface_mac)
        .await;
    match (
        setup_result,
        expected_dpu_count,
        site_config.site_explorer.allow_zero_dpu_hosts,
    ) {
        (Err(RedfishError::NoDpu), 0, true) => {
            tracing::info!(
                "redfish set_boot_order_dpu_first failed due to there being no DPUs on the host. This is expected as the host has no DPUs, and we are configured to allow this."
            );
            Ok(None)
        }
        (Ok(job_id), _, _) => Ok(job_id),
        (Err(e), _, _) => Err(e),
    }
}

// Returns true if update_manager flagged this managed host as needing its firmware examined
pub(super) async fn is_machine_validation_requested(state: &ManagedHostStateSnapshot) -> bool {
    let Some(on_demand_machine_validation_request) =
        state.host_snapshot.on_demand_machine_validation_request
    else {
        return false;
    };

    if on_demand_machine_validation_request {
        tracing::info!(machine_id = %state.host_snapshot.id, "Machine Validation is requested");
    }

    on_demand_machine_validation_request
}

pub(super) async fn log_host_config(
    redfish_client: &dyn Redfish,
    mh_snapshot: &ManagedHostStateSnapshot,
) {
    let host_id = mh_snapshot.host_snapshot.id;
    let managed_state = &mh_snapshot.managed_state;

    let boot_options = match redfish_client.get_boot_options().await {
        Ok(opts) => opts,
        Err(e) => {
            tracing::warn!(
                %host_id,
                %managed_state,
                error = %e,
                "Failed to fetch boot options"
            );
            return;
        }
    };

    let mut boot_entries = Vec::with_capacity(boot_options.members.len());
    for (i, member) in boot_options.members.iter().enumerate() {
        let option_id = member.odata_id.split('/').next_back().unwrap_or("unknown");
        match redfish_client.get_boot_option(option_id).await {
            Ok(opt) => {
                let enabled = match opt.boot_option_enabled {
                    Some(true) => "enabled",
                    Some(false) => "disabled",
                    None => "unknown",
                };
                let device_path = opt.uefi_device_path.as_deref().unwrap_or("N/A");
                boot_entries.push(format!(
                    "  #{}: {} (id={}, {}, path={})",
                    i + 1,
                    opt.display_name,
                    opt.id,
                    enabled,
                    device_path
                ));
            }
            Err(e) => {
                boot_entries.push(format!(
                    "  #{}: {} (failed to fetch details: {})",
                    i + 1,
                    option_id,
                    e
                ));
            }
        }
    }

    let pcie_section = match redfish_client.pcie_devices().await {
        Ok(devices) => {
            let entries: Vec<String> = devices
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    format!(
                        "  #{}: {} (id={}, manufacturer={}, part={}, serial={}, fw={})",
                        i + 1,
                        dev.name.as_deref().unwrap_or("N/A"),
                        dev.id.as_deref().unwrap_or("N/A"),
                        dev.manufacturer.as_deref().unwrap_or("N/A"),
                        dev.part_number.as_deref().unwrap_or("N/A"),
                        dev.serial_number.as_deref().unwrap_or("N/A"),
                        dev.firmware_version.as_deref().unwrap_or("N/A"),
                    )
                })
                .collect();
            format!("PCIe devices:\n{}", entries.join("\n"))
        }
        Err(e) => format!("PCIe devices: failed to fetch ({})", e),
    };

    tracing::info!(
        %host_id,
        %managed_state,
        "Host config:\nBoot order:\n{}\n{}",
        boot_entries.join("\n"),
        pcie_section
    );
}

pub(super) async fn configure_host_bios(
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    reachability_params: &ReachabilityParams,
    redfish_client: &dyn Redfish,
    mh_snapshot: &ManagedHostStateSnapshot,
) -> Result<BiosConfigOutcome, StateHandlerError> {
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
        // This is the Zero-DPU case
        None
    };

    if let Err(e) = call_machine_setup_and_handle_no_dpu_error(
        redfish_client,
        boot_interface_mac.as_deref(),
        mh_snapshot.host_snapshot.associated_dpu_machine_ids().len(),
        &ctx.services.site_config,
    )
    .await
    {
        tracing::warn!(
            "redfish machine_setup failed for {}, potentially due to known race condition between UEFI POST and BMC. triggering force-restart if needed. err: {}",
            mh_snapshot.host_snapshot.id,
            e
        );

        // if machine_setup failed, rebooted to potentially work around
        // a known race between the DPU UEFI and the BMC, where if
        // the BMC is not up when DPU UEFI runs, then Attributes might
        // not come through. The fix is to force-restart the DPU to
        // re-POST.
        //
        // As of July 2024, Josh Price said there's an NBU FR to fix
        // this, but it wasn't target to a release yet.
        let reboot_status = if mh_snapshot.host_snapshot.last_reboot_requested.is_none() {
            handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart).await?;

            RebootStatus {
                increase_retry_count: true,
                status: "Restarted host".to_string(),
            }
        } else {
            trigger_reboot_if_needed(
                &mh_snapshot.host_snapshot,
                mh_snapshot,
                None,
                reachability_params,
                ctx,
            )
            .await?
        };
        // Return WaitingForReboot instead of Err to ensure the transaction is committed
        // and last_reboot_requested is persisted. Returning Err would cause a transaction
        // rollback, leading to a tight reboot loop since the reboot timestamp is lost.
        return Ok(BiosConfigOutcome::WaitingForReboot(format!(
            "redfish machine_setup failed: {e}; triggered host reboot: {reboot_status:#?}"
        )));
    };

    // Host needs to be rebooted to pick up the changes after calling machine_setup
    handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart).await?;
    Ok(BiosConfigOutcome::Done)
}

pub(super) async fn set_host_boot_order(
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    reachability_params: &ReachabilityParams,
    redfish_client: &dyn Redfish,
    mh_snapshot: &ManagedHostStateSnapshot,
    set_boot_order_info: SetBootOrderInfo,
) -> Result<SetBootOrderOutcome, StateHandlerError> {
    match set_boot_order_info.set_boot_order_state {
        SetBootOrderState::SetBootOrder => {
            if mh_snapshot.dpu_snapshots.is_empty() {
                // MachineState::SetBootOrder is a NO-OP for the Zero-DPU case
                Ok(SetBootOrderOutcome::Done)
            } else {
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

                let jid = match set_boot_order_dpu_first_and_handle_no_dpu_error(
                    redfish_client,
                    &primary_interface.mac_address.to_string(),
                    mh_snapshot.host_snapshot.associated_dpu_machine_ids().len(),
                    &ctx.services.site_config,
                )
                .await
                {
                    Ok(jid) => jid,
                    Err(e) => {
                        tracing::warn!(
                            "redfish set_boot_order_dpu_first failed for {}, potentially due to known race condition between UEFI POST and BMC. triggering force-restart if needed. err: {}",
                            mh_snapshot.host_snapshot.id,
                            e
                        );

                        let reboot_status =
                            if mh_snapshot.host_snapshot.last_reboot_requested.is_none() {
                                handler_host_power_control(
                                    mh_snapshot,
                                    ctx,
                                    SystemPowerControl::ForceRestart,
                                )
                                .await?;

                                RebootStatus {
                                    increase_retry_count: true,
                                    status: "Restarted host".to_string(),
                                }
                            } else {
                                trigger_reboot_if_needed(
                                    &mh_snapshot.host_snapshot,
                                    mh_snapshot,
                                    None,
                                    reachability_params,
                                    ctx,
                                )
                                .await?
                            };

                        // Log boot options and PCIe device list whenever a fresh reboot is
                        // triggered so we capture full diagnostic context (UEFI device paths +
                        // PCIe inventory) before state resets. Skipped when waiting on an
                        // already-in-progress reboot to avoid redundant Redfish calls.
                        if reboot_status.increase_retry_count {
                            log_host_config(redfish_client, mh_snapshot).await;
                        }

                        // Return wait instead of Err to ensure the transaction is committed
                        // and last_reboot_requested is persisted. Returning Err would cause a transaction
                        // rollback, leading to a tight reboot loop since the reboot timestamp is lost.
                        return Ok(SetBootOrderOutcome::WaitingForReboot(format!(
                            "redfish set_boot_order_dpu_first failed: {e}; triggered host reboot: {reboot_status:#?}"
                        )));
                    }
                };

                Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                    set_boot_order_jid: jid,
                    set_boot_order_state: SetBootOrderState::WaitForSetBootOrderJobScheduled,
                    retry_count: set_boot_order_info.retry_count,
                }))
            }
        }
        SetBootOrderState::WaitForSetBootOrderJobScheduled => {
            if let Some(job_id) = &set_boot_order_info.set_boot_order_jid {
                let job_state = redfish_client.get_job_state(job_id).await.map_err(|e| {
                    StateHandlerError::RedfishError {
                        operation: "get_job_state",
                        error: e,
                    }
                })?;

                if !matches!(job_state, libredfish::JobState::Scheduled) {
                    return Err(StateHandlerError::GenericError(eyre::eyre!(
                        "waiting for job {:#?} to be scheduled; current state: {job_state:#?}",
                        job_id
                    )));
                }
            }

            Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                set_boot_order_jid: set_boot_order_info.set_boot_order_jid.clone(),
                set_boot_order_state: SetBootOrderState::RebootHost,
                retry_count: set_boot_order_info.retry_count,
            }))
        }
        SetBootOrderState::RebootHost => {
            // Host needs to be rebooted to pick up the changes after calling machine_setup
            handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart).await?;

            Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                set_boot_order_jid: set_boot_order_info.set_boot_order_jid.clone(),
                set_boot_order_state: SetBootOrderState::WaitForSetBootOrderJobCompletion,
                retry_count: set_boot_order_info.retry_count,
            }))
        }
        SetBootOrderState::WaitForSetBootOrderJobCompletion => {
            const JOB_QUERY_WAIT_MINUTES: i64 = 5;

            if let Some(job_id) = &set_boot_order_info.set_boot_order_jid {
                let job_state = match redfish_client.get_job_state(job_id).await {
                    Ok(state) => state,
                    Err(e) => {
                        // Wait 5 minutes before declaring the job was lost or failed.
                        // This helps differentiate between transient errors and true failures.
                        let minutes_since_state_change = mh_snapshot
                            .host_snapshot
                            .state
                            .version
                            .since_state_change()
                            .num_minutes();

                        if minutes_since_state_change < JOB_QUERY_WAIT_MINUTES {
                            return Err(StateHandlerError::RedfishError {
                                operation: "get_job_state",
                                error: e,
                            });
                        }

                        tracing::warn!(
                            "SetBootOrder: job {} lookup failed for {} after {} minutes, transitioning to HandleJobFailure: {}",
                            job_id,
                            mh_snapshot.host_snapshot.id,
                            minutes_since_state_change,
                            e
                        );

                        return Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                            set_boot_order_jid: None,
                            set_boot_order_state: SetBootOrderState::HandleJobFailure {
                                failure: format!("Job {} lookup failed: {}", job_id, e),
                                power_state: libredfish::PowerState::Off,
                            },
                            retry_count: set_boot_order_info.retry_count,
                        }));
                    }
                };

                match job_state {
                    libredfish::JobState::Completed => {
                        // Job completed successfully, proceed to CheckBootOrder
                    }
                    libredfish::JobState::ScheduledWithErrors
                    | libredfish::JobState::CompletedWithErrors => {
                        tracing::warn!(
                            "SetBootOrder: job {} failed for {} with state {job_state:#?}, transitioning to HandleJobFailure",
                            job_id,
                            mh_snapshot.host_snapshot.id,
                        );

                        return Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                            set_boot_order_jid: None,
                            set_boot_order_state: SetBootOrderState::HandleJobFailure {
                                failure: format!("Job {} failed: {job_state:#?}", job_id),
                                power_state: libredfish::PowerState::Off,
                            },
                            retry_count: set_boot_order_info.retry_count,
                        }));
                    }
                    _ => {
                        // Job is still running, wait for completion
                        return Err(StateHandlerError::GenericError(eyre::eyre!(
                            "waiting for job {:#?} to complete; current state: {job_state:#?}",
                            job_id
                        )));
                    }
                }
            }

            Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                set_boot_order_jid: set_boot_order_info.set_boot_order_jid.clone(),
                set_boot_order_state: SetBootOrderState::CheckBootOrder,
                retry_count: set_boot_order_info.retry_count,
            }))
        }
        SetBootOrderState::HandleJobFailure {
            failure,
            power_state,
        } => {
            // Handles recovery when a SetBootOrder BIOS job fails or is lost.
            // 1. Power off the host
            // 2. Reset the BMC
            // 3. Transition to CheckBootOrder to verify and retry if needed

            let current_power_state = redfish_client.get_power_state().await.map_err(|e| {
                StateHandlerError::RedfishError {
                    operation: "get_power_state",
                    error: e,
                }
            })?;

            match power_state {
                libredfish::PowerState::Off => {
                    if current_power_state != libredfish::PowerState::Off {
                        handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceOff)
                            .await?;

                        return Ok(SetBootOrderOutcome::WaitingForReboot(format!(
                            "HandleJobFailure: waiting for {} to power down; current power state: {current_power_state}; failure: {}",
                            mh_snapshot.host_snapshot.id, failure
                        )));
                    }

                    // Host is powered off, reset the BMC
                    tracing::info!(
                        "HandleJobFailure: Resetting BMC for {} after failure: {}",
                        mh_snapshot.host_snapshot.id,
                        failure
                    );

                    redfish_client.bmc_reset().await.map_err(|e| {
                        StateHandlerError::RedfishError {
                            operation: "bmc_reset",
                            error: e,
                        }
                    })?;

                    // Transition to PowerState::On to wait for BMC to come back
                    Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                        set_boot_order_jid: None,
                        set_boot_order_state: SetBootOrderState::HandleJobFailure {
                            failure: failure.clone(),
                            power_state: libredfish::PowerState::On,
                        },
                        retry_count: set_boot_order_info.retry_count,
                    }))
                }
                libredfish::PowerState::On => {
                    // BMC should be back, power the host back on
                    if current_power_state != libredfish::PowerState::On {
                        // Wait for the BMC to come back online after reset before powering on
                        let basetime = mh_snapshot
                            .host_snapshot
                            .last_reboot_requested
                            .as_ref()
                            .map(|x| x.time)
                            .unwrap_or(mh_snapshot.host_snapshot.state.version.timestamp());

                        let power_down_wait = ctx
                            .services
                            .site_config
                            .machine_state_controller
                            .power_down_wait;

                        if Utc::now().signed_duration_since(basetime) < power_down_wait {
                            return Ok(SetBootOrderOutcome::WaitingForReboot(format!(
                                "HandleJobFailure: waiting for BMC to come back online for {}; job failure: {}",
                                mh_snapshot.host_snapshot.id, failure
                            )));
                        }

                        handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::On)
                            .await?;

                        return Ok(SetBootOrderOutcome::WaitingForReboot(format!(
                            "HandleJobFailure: powering on {} after BMC reset; job failure: {}",
                            mh_snapshot.host_snapshot.id, failure
                        )));
                    }

                    // Host is powered on, transition to CheckBootOrder to verify and retry
                    tracing::info!(
                        "HandleJobFailure: BMC reset complete and host powered on for {}, transitioning to CheckBootOrder",
                        mh_snapshot.host_snapshot.id,
                    );

                    Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                        set_boot_order_jid: None,
                        set_boot_order_state: SetBootOrderState::CheckBootOrder,
                        retry_count: set_boot_order_info.retry_count,
                    }))
                }
                _ => Err(StateHandlerError::GenericError(eyre::eyre!(
                    "HandleJobFailure: unexpected power state {power_state:#?} for {}",
                    mh_snapshot.host_snapshot.id
                ))),
            }
        }
        SetBootOrderState::CheckBootOrder => {
            const MAX_BOOT_ORDER_RETRIES: u32 = 3;
            const CHECK_BOOT_ORDER_TIMEOUT_MINUTES: i64 = 30;

            let retry_count = set_boot_order_info.retry_count;

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

            let boot_order_configured = redfish_client
                .is_boot_order_setup(&primary_interface.mac_address.to_string())
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "is_boot_order_setup",
                    error: e,
                })?;

            if boot_order_configured {
                tracing::info!(
                    "Boot order verified for {} - the host has its boot order configured properly",
                    mh_snapshot.host_snapshot.id,
                );
                return Ok(SetBootOrderOutcome::Done);
            }

            // Boot order is not configured properly - check if we should retry
            let time_since_state_change =
                mh_snapshot.host_snapshot.state.version.since_state_change();

            tracing::warn!(
                "Boot order check failed for {} - the host does not have its boot order configured properly after SetBootOrder (retry_count: {}, time_in_state: {} minutes)",
                mh_snapshot.host_snapshot.id,
                retry_count,
                time_since_state_change.num_minutes()
            );

            // If we've been stuck for 30+ minutes and haven't exhausted retries, retry SetBootOrder
            if time_since_state_change.num_minutes() >= CHECK_BOOT_ORDER_TIMEOUT_MINUTES
                && retry_count < MAX_BOOT_ORDER_RETRIES
            {
                tracing::info!(
                    "Boot order check timed out for {} after {} minutes, retrying SetBootOrder (retry {} of {})",
                    mh_snapshot.host_snapshot.id,
                    time_since_state_change.num_minutes(),
                    retry_count + 1,
                    MAX_BOOT_ORDER_RETRIES
                );

                return Ok(SetBootOrderOutcome::Continue(SetBootOrderInfo {
                    set_boot_order_jid: None,
                    set_boot_order_state: SetBootOrderState::SetBootOrder,
                    retry_count: retry_count + 1,
                }));
            }

            // Either still within timeout window or exhausted retries - return error
            Err(StateHandlerError::GenericError(eyre::eyre!(
                "Boot order is not configured properly for host {} after SetBootOrder completed (retry_count: {}, time_in_state: {} minutes)",
                mh_snapshot.host_snapshot.id,
                retry_count,
                time_since_state_change.num_minutes()
            )))
        }
    }
}

pub(super) async fn handle_bfb_install_state(
    state: &ManagedHostStateSnapshot,
    substate: InstallDpuOsState,
    dpu_snapshot: &Machine,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    next_state_resolver: &impl NextState,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let dpu_machine_id = &dpu_snapshot.id.clone();
    let dpu_redfish_client_result = ctx
        .services
        .create_redfish_client_from_machine(dpu_snapshot)
        .await;

    let dpu_redfish_client = match dpu_redfish_client_result {
        Ok(redfish_client) => redfish_client,
        Err(e) => {
            return Ok(StateHandlerOutcome::wait(format!(
                "Waiting for RedFish to become available: {:?}",
                e
            )));
        }
    };
    match substate {
        InstallDpuOsState::Completed => Ok(StateHandlerOutcome::transition(
            next_state_resolver.next_bfb_install_state(
                &state.managed_state,
                &InstallDpuOsState::Completed,
                dpu_machine_id,
            )?,
        )),
        InstallDpuOsState::InstallationError { .. } => Ok(StateHandlerOutcome::do_nothing()),

        InstallDpuOsState::InstallingBFB => {
            let task = dpu_redfish_client
                .update_firmware_simple_update(
                    "carbide-pxe.forge//public/blobs/internal/aarch64/forge.bfb",
                    vec!["redfish/v1/UpdateService/FirmwareInventory/DPU_OS".to_string()],
                    TransferProtocolType::HTTP,
                )
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "update_firmware_simple_update",
                    error: e,
                })?;
            tracing::info!(
                "DPU {} OS install task {} submitted.",
                dpu_snapshot.id,
                task.id
            );
            Ok(StateHandlerOutcome::transition(
                next_state_resolver.next_bfb_install_state(
                    &state.managed_state,
                    &InstallDpuOsState::WaitForInstallComplete {
                        task_id: task.id,
                        progress: "0".to_string(),
                    },
                    dpu_machine_id,
                )?,
            ))
        }

        InstallDpuOsState::WaitForInstallComplete { task_id, .. } => {
            let task = dpu_redfish_client
                .get_task(task_id.as_str())
                .await
                .map_err(|e| StateHandlerError::RedfishError {
                    operation: "get_task",
                    error: e,
                })?;

            tracing::info!(
                "DPU {} OS install task {}: {:#?}",
                dpu_snapshot.id,
                task.id,
                task.task_state
            );

            match task.task_state {
                Some(TaskState::Completed) => {
                    tracing::info!("Install BFB on {:#?} completed", dpu_snapshot.bmc_addr());
                    let next_state = next_state_resolver.next_bfb_install_state(
                        &state.managed_state,
                        &InstallDpuOsState::Completed,
                        dpu_machine_id,
                    )?;
                    Ok(StateHandlerOutcome::transition(next_state))
                }
                Some(TaskState::Exception) => {
                    let msg = format!(
                        "BFB install task {} on {:#?} failed: {}.",
                        task_id,
                        dpu_snapshot.bmc_addr(),
                        task.messages.iter().map(|t| t.message.clone()).join("\n")
                    );
                    tracing::error!(msg);
                    let next_state = next_state_resolver.next_bfb_install_state(
                        &state.managed_state,
                        &InstallDpuOsState::InstallationError { msg },
                        dpu_machine_id,
                    )?;
                    Ok(StateHandlerOutcome::transition(next_state))
                }
                Some(TaskState::Running) | Some(TaskState::New) | Some(TaskState::Starting) => {
                    let percent_complete = task
                        .percent_complete
                        .map_or("0".to_string(), |p| p.to_string());
                    Ok(StateHandlerOutcome::wait(format!(
                        "Waiting for BFB install to complete: {}%",
                        percent_complete
                    )))
                }
                task_state => {
                    let msg = format!(
                        "BFB install task {} on {:#?} failed ({:#?}): {}",
                        task_id,
                        dpu_snapshot.bmc_addr(),
                        task_state,
                        task.messages.iter().map(|t| t.message.clone()).join("\n")
                    );
                    tracing::error!(msg);
                    let next_state = next_state_resolver.next_bfb_install_state(
                        &state.managed_state,
                        &InstallDpuOsState::InstallationError { msg },
                        dpu_machine_id,
                    )?;
                    Ok(StateHandlerOutcome::transition(next_state))
                }
            }
        }
    }
}
