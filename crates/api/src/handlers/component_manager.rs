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

use std::collections::HashMap;
use std::net::IpAddr;

use ::rpc::common::SystemPowerControl;
use ::rpc::forge as rpc;
use carbide_uuid::power_shelf::PowerShelfId;
use carbide_uuid::switch::SwitchId;
use component_manager::component_manager::ComponentManager;
use component_manager::error::ComponentManagerError;
use component_manager::nv_switch_manager::SwitchEndpoint;
use component_manager::power_shelf_manager::{PowerShelfEndpoint, PowerShelfVendor};
use component_manager::types::PowerAction;
use db;
use mac_address::MacAddress;
use tonic::{Request, Response, Status};

use crate::api::{Api, log_request_data};

fn require_component_manager(api: &Api) -> Result<&ComponentManager, Status> {
    api.component_manager
        .as_ref()
        .ok_or_else(|| Status::unimplemented("component manager is not configured"))
}

fn component_manager_error_to_status(err: ComponentManagerError) -> Status {
    match err {
        ComponentManagerError::Unavailable(msg) => Status::unavailable(msg),
        ComponentManagerError::NotFound(msg) => Status::not_found(msg),
        ComponentManagerError::InvalidArgument(msg) => Status::invalid_argument(msg),
        ComponentManagerError::Internal(msg) => Status::internal(msg),
        ComponentManagerError::Transport(e) => Status::unavailable(format!("transport error: {e}")),
        ComponentManagerError::Status(s) => s,
    }
}

fn make_result(
    id: &str,
    status: rpc::ComponentManagerStatusCode,
    error: Option<String>,
) -> rpc::ComponentResult {
    rpc::ComponentResult {
        component_id: id.to_owned(),
        status: status as i32,
        error: error.unwrap_or_default(),
    }
}

fn success_result(id: &str) -> rpc::ComponentResult {
    make_result(id, rpc::ComponentManagerStatusCode::Success, None)
}

fn not_found_result(id: &str) -> rpc::ComponentResult {
    make_result(
        id,
        rpc::ComponentManagerStatusCode::NotFound,
        Some(format!("no explored endpoint found for {id}")),
    )
}

fn error_result(id: &str, error: String) -> rpc::ComponentResult {
    make_result(
        id,
        rpc::ComponentManagerStatusCode::InternalError,
        Some(error),
    )
}

fn build_inventory_entries(
    id_strings: &[String],
    report_by_id: &HashMap<String, model::site_explorer::EndpointExplorationReport>,
) -> Vec<rpc::ComponentInventoryEntry> {
    id_strings
        .iter()
        .map(|id| match report_by_id.get(id) {
            Some(report) => rpc::ComponentInventoryEntry {
                result: Some(success_result(id)),
                report: Some(report.clone().into()),
            },
            None => rpc::ComponentInventoryEntry {
                result: Some(not_found_result(id)),
                report: None,
            },
        })
        .collect()
}

fn map_power_action(raw: i32) -> Result<PowerAction, Status> {
    match SystemPowerControl::try_from(raw) {
        Ok(SystemPowerControl::On) => Ok(PowerAction::On),
        Ok(SystemPowerControl::GracefulShutdown) => Ok(PowerAction::GracefulShutdown),
        Ok(SystemPowerControl::ForceOff) => Ok(PowerAction::ForceOff),
        Ok(SystemPowerControl::GracefulRestart) => Ok(PowerAction::GracefulRestart),
        Ok(SystemPowerControl::ForceRestart) => Ok(PowerAction::ForceRestart),
        Ok(SystemPowerControl::AcPowercycle) => Ok(PowerAction::AcPowercycle),
        Err(_) => Err(Status::invalid_argument(format!(
            "unknown power action: {raw}"
        ))),
    }
}

// ---- Endpoint resolution helpers ----

struct ResolvedSwitches {
    endpoints: Vec<SwitchEndpoint>,
    mac_to_id: HashMap<MacAddress, SwitchId>,
}

async fn resolve_switch_endpoints(
    api: &Api,
    switch_ids: &[SwitchId],
) -> Result<ResolvedSwitches, Status> {
    let rows = db::switch::find_switch_endpoints_by_ids(&mut api.db_reader(), switch_ids)
        .await
        .map_err(|e| Status::internal(format!("db error resolving switch endpoints: {e}")))?;

    let mut endpoints = Vec::with_capacity(rows.len());
    let mut mac_to_id = HashMap::with_capacity(rows.len());
    let mut skipped = Vec::new();

    for row in rows {
        let (Some(nvos_mac), Some(nvos_ip)) = (row.nvos_mac, row.nvos_ip) else {
            tracing::warn!(
                switch_id = %row.switch_id,
                "skipping switch: NVOS MAC or IP not available"
            );
            skipped.push(row.switch_id);
            continue;
        };
        mac_to_id.insert(row.bmc_mac, row.switch_id);
        endpoints.push(SwitchEndpoint {
            bmc_ip: row.bmc_ip,
            bmc_mac: row.bmc_mac,
            nvos_ip,
            nvos_mac,
        });
    }

    if !skipped.is_empty() {
        tracing::warn!(
            count = skipped.len(),
            "some switches were skipped due to missing NVOS info"
        );
    }

    Ok(ResolvedSwitches {
        endpoints,
        mac_to_id,
    })
}

struct ResolvedPowerShelves {
    endpoints: Vec<PowerShelfEndpoint>,
    mac_to_id: HashMap<MacAddress, PowerShelfId>,
}

async fn resolve_power_shelf_endpoints(
    api: &Api,
    power_shelf_ids: &[PowerShelfId],
) -> Result<ResolvedPowerShelves, Status> {
    let rows =
        db::power_shelf::find_power_shelf_endpoints_by_ids(&mut api.db_reader(), power_shelf_ids)
            .await
            .map_err(|e| {
                Status::internal(format!("db error resolving power shelf endpoints: {e}"))
            })?;

    let mut endpoints = Vec::with_capacity(rows.len());
    let mut mac_to_id = HashMap::with_capacity(rows.len());

    for row in rows {
        mac_to_id.insert(row.pmc_mac, row.power_shelf_id);
        endpoints.push(PowerShelfEndpoint {
            pmc_ip: row.pmc_ip,
            pmc_mac: row.pmc_mac,
            // TODO: retrieve vendor from DB instead of using a hardcoded default
            pmc_vendor: PowerShelfVendor::DEFAULT,
        });
    }

    Ok(ResolvedPowerShelves {
        endpoints,
        mac_to_id,
    })
}

fn switch_mac_to_id_str(mac: &MacAddress, mac_to_id: &HashMap<MacAddress, SwitchId>) -> String {
    mac_to_id
        .get(mac)
        .map(|id| id.to_string())
        .unwrap_or_else(|| mac.to_string())
}

fn ps_mac_to_id_str(mac: &MacAddress, mac_to_id: &HashMap<MacAddress, PowerShelfId>) -> String {
    mac_to_id
        .get(mac)
        .map(|id| id.to_string())
        .unwrap_or_else(|| mac.to_string())
}

fn map_fw_state(state: component_manager::types::FirmwareState) -> i32 {
    use component_manager::types::FirmwareState;
    match state {
        FirmwareState::Unknown => rpc::FirmwareUpdateState::FwStateUnknown as i32,
        FirmwareState::Queued => rpc::FirmwareUpdateState::FwStateQueued as i32,
        FirmwareState::InProgress => rpc::FirmwareUpdateState::FwStateInProgress as i32,
        FirmwareState::Verifying => rpc::FirmwareUpdateState::FwStateVerifying as i32,
        FirmwareState::Completed => rpc::FirmwareUpdateState::FwStateCompleted as i32,
        FirmwareState::Failed => rpc::FirmwareUpdateState::FwStateFailed as i32,
        FirmwareState::Cancelled => rpc::FirmwareUpdateState::FwStateCancelled as i32,
    }
}

// ---- Power Control ----

pub(crate) async fn component_power_control(
    api: &Api,
    request: Request<rpc::ComponentPowerControlRequest>,
) -> Result<Response<rpc::ComponentPowerControlResponse>, Status> {
    log_request_data(&request);
    let cm = require_component_manager(api)?;
    let req = request.into_inner();

    let action = map_power_action(req.action)?;

    let target = req
        .target
        .ok_or_else(|| Status::invalid_argument("target is required"))?;

    let results = match target {
        rpc::component_power_control_request::Target::SwitchIds(list) => {
            let resolved = resolve_switch_endpoints(api, &list.ids).await?;
            tracing::info!(
                backend = cm.nv_switch.name(),
                count = resolved.endpoints.len(),
                ?action,
                "power control for switches"
            );
            let backend_results = cm
                .nv_switch
                .power_control(&resolved.endpoints, action)
                .await
                .map_err(component_manager_error_to_status)?;
            backend_results
                .into_iter()
                .map(|r| {
                    let id = switch_mac_to_id_str(&r.bmc_mac, &resolved.mac_to_id);
                    if r.success {
                        success_result(&id)
                    } else {
                        error_result(&id, r.error.unwrap_or_default())
                    }
                })
                .collect()
        }
        rpc::component_power_control_request::Target::PowerShelfIds(list) => {
            let resolved = resolve_power_shelf_endpoints(api, &list.ids).await?;
            tracing::info!(
                backend = cm.power_shelf.name(),
                count = resolved.endpoints.len(),
                ?action,
                "power control for power shelves"
            );
            let backend_results = cm
                .power_shelf
                .power_control(&resolved.endpoints, action)
                .await
                .map_err(component_manager_error_to_status)?;
            backend_results
                .into_iter()
                .map(|r| {
                    let id = ps_mac_to_id_str(&r.pmc_mac, &resolved.mac_to_id);
                    if r.success {
                        success_result(&id)
                    } else {
                        error_result(&id, r.error.unwrap_or_default())
                    }
                })
                .collect()
        }
        rpc::component_power_control_request::Target::MachineIds(_list) => {
            return Err(Status::unimplemented(
                "machine power control should use AdminPowerControl",
            ));
        }
    };

    Ok(Response::new(rpc::ComponentPowerControlResponse {
        results,
    }))
}

// ---- Inventory ----

pub(crate) async fn get_component_inventory(
    api: &Api,
    request: Request<rpc::GetComponentInventoryRequest>,
) -> Result<Response<rpc::GetComponentInventoryResponse>, Status> {
    log_request_data(&request);
    let req = request.into_inner();

    let target = req
        .target
        .ok_or_else(|| Status::invalid_argument("target is required"))?;

    let entries = match target {
        rpc::get_component_inventory_request::Target::SwitchIds(list) => {
            let id_ip_pairs =
                db::switch::find_bmc_ips_by_switch_ids(&mut api.db_reader(), &list.ids)
                    .await
                    .map_err(|e| Status::internal(format!("db error: {e}")))?;

            let ip_to_id: HashMap<IpAddr, String> = id_ip_pairs
                .into_iter()
                .map(|(sid, ip)| (ip, sid.to_string()))
                .collect();

            let id_strings: Vec<String> = list.ids.iter().map(|id| id.to_string()).collect();
            let ips: Vec<IpAddr> = ip_to_id.keys().copied().collect();
            let endpoints = db::explored_endpoints::find_by_ips(&mut api.db_reader(), ips)
                .await
                .map_err(|e| Status::internal(format!("db error: {e}")))?;

            let report_by_id: HashMap<String, _> = endpoints
                .into_iter()
                .filter_map(|ep| {
                    let id = ip_to_id.get(&ep.address)?;
                    Some((id.clone(), ep.report))
                })
                .collect();

            build_inventory_entries(&id_strings, &report_by_id)
        }
        rpc::get_component_inventory_request::Target::PowerShelfIds(list) => {
            let id_ip_pairs =
                db::power_shelf::find_bmc_ips_by_power_shelf_ids(&mut api.db_reader(), &list.ids)
                    .await
                    .map_err(|e| Status::internal(format!("db error: {e}")))?;

            let ip_to_id: HashMap<IpAddr, String> = id_ip_pairs
                .into_iter()
                .map(|(psid, ip)| (ip, psid.to_string()))
                .collect();

            let id_strings: Vec<String> = list.ids.iter().map(|id| id.to_string()).collect();
            let ips: Vec<IpAddr> = ip_to_id.keys().copied().collect();
            let endpoints = db::explored_endpoints::find_by_ips(&mut api.db_reader(), ips)
                .await
                .map_err(|e| Status::internal(format!("db error: {e}")))?;

            let report_by_id: HashMap<String, _> = endpoints
                .into_iter()
                .filter_map(|ep| {
                    let id = ip_to_id.get(&ep.address)?;
                    Some((id.clone(), ep.report))
                })
                .collect();

            build_inventory_entries(&id_strings, &report_by_id)
        }
        rpc::get_component_inventory_request::Target::MachineIds(list) => {
            let id_strings: Vec<String> =
                list.machine_ids.iter().map(|id| id.to_string()).collect();

            let mut txn = api
                .txn_begin()
                .await
                .map_err(|e| Status::internal(format!("db error: {e}")))?;

            let bmc_pairs = db::machine_topology::find_machine_bmc_pairs_by_machine_id(
                &mut txn,
                list.machine_ids.clone(),
            )
            .await
            .map_err(|e| Status::internal(format!("db error: {e}")))?;

            txn.commit()
                .await
                .map_err(|e| Status::internal(format!("db error: {e}")))?;

            let ip_to_id: HashMap<IpAddr, String> = bmc_pairs
                .into_iter()
                .filter_map(|(mid, ip_str)| {
                    let ip: IpAddr = ip_str?.parse().ok()?;
                    Some((ip, mid.to_string()))
                })
                .collect();

            let ips: Vec<IpAddr> = ip_to_id.keys().copied().collect();
            let endpoints = db::explored_endpoints::find_by_ips(&mut api.db_reader(), ips)
                .await
                .map_err(|e| Status::internal(format!("db error: {e}")))?;

            let report_by_id: HashMap<String, _> = endpoints
                .into_iter()
                .filter_map(|ep| {
                    let id = ip_to_id.get(&ep.address)?;
                    Some((id.clone(), ep.report))
                })
                .collect();

            build_inventory_entries(&id_strings, &report_by_id)
        }
    };

    Ok(Response::new(rpc::GetComponentInventoryResponse {
        entries,
    }))
}

// ---- Firmware Update ----

pub(crate) async fn update_component_firmware(
    api: &Api,
    request: Request<rpc::UpdateComponentFirmwareRequest>,
) -> Result<Response<rpc::UpdateComponentFirmwareResponse>, Status> {
    log_request_data(&request);
    let cm = require_component_manager(api)?;
    let req = request.into_inner();

    let target = req
        .target
        .ok_or_else(|| Status::invalid_argument("target is required"))?;

    let results = match target {
        rpc::update_component_firmware_request::Target::SwitchIds(list) => {
            let resolved = resolve_switch_endpoints(api, &list.ids).await?;
            let backend_results = cm
                .nv_switch
                .queue_firmware_updates(&resolved.endpoints, &req.target_version, &req.components)
                .await
                .map_err(component_manager_error_to_status)?;
            backend_results
                .into_iter()
                .map(|r| {
                    let id = switch_mac_to_id_str(&r.bmc_mac, &resolved.mac_to_id);
                    if r.success {
                        success_result(&id)
                    } else {
                        error_result(&id, r.error.unwrap_or_default())
                    }
                })
                .collect()
        }
        rpc::update_component_firmware_request::Target::PowerShelfIds(list) => {
            let resolved = resolve_power_shelf_endpoints(api, &list.ids).await?;
            let backend_results = cm
                .power_shelf
                .update_firmware(&resolved.endpoints, &req.target_version, &req.components)
                .await
                .map_err(component_manager_error_to_status)?;
            backend_results
                .into_iter()
                .map(|r| {
                    let id = ps_mac_to_id_str(&r.pmc_mac, &resolved.mac_to_id);
                    if r.success {
                        success_result(&id)
                    } else {
                        error_result(&id, r.error.unwrap_or_default())
                    }
                })
                .collect()
        }
        rpc::update_component_firmware_request::Target::MachineIds(_) => {
            return Err(Status::unimplemented(
                "machine firmware updates are not supported via this RPC",
            ));
        }
    };

    Ok(Response::new(rpc::UpdateComponentFirmwareResponse {
        results,
    }))
}

// ---- Firmware Status ----

pub(crate) async fn get_component_firmware_status(
    api: &Api,
    request: Request<rpc::GetComponentFirmwareStatusRequest>,
) -> Result<Response<rpc::GetComponentFirmwareStatusResponse>, Status> {
    log_request_data(&request);
    let cm = require_component_manager(api)?;
    let req = request.into_inner();

    let target = req
        .target
        .ok_or_else(|| Status::invalid_argument("target is required"))?;

    let statuses = match target {
        rpc::get_component_firmware_status_request::Target::SwitchIds(list) => {
            let resolved = resolve_switch_endpoints(api, &list.ids).await?;
            let backend_statuses = cm
                .nv_switch
                .get_firmware_status(&resolved.endpoints)
                .await
                .map_err(component_manager_error_to_status)?;
            backend_statuses
                .into_iter()
                .map(|s| {
                    let id = switch_mac_to_id_str(&s.bmc_mac, &resolved.mac_to_id);
                    rpc::FirmwareUpdateStatus {
                        result: Some(if s.error.is_none() {
                            success_result(&id)
                        } else {
                            error_result(&id, s.error.unwrap_or_default())
                        }),
                        state: map_fw_state(s.state),
                        target_version: s.target_version,
                        updated_at: None,
                    }
                })
                .collect()
        }
        rpc::get_component_firmware_status_request::Target::PowerShelfIds(list) => {
            let resolved = resolve_power_shelf_endpoints(api, &list.ids).await?;
            let backend_statuses = cm
                .power_shelf
                .get_firmware_status(&resolved.endpoints)
                .await
                .map_err(component_manager_error_to_status)?;
            backend_statuses
                .into_iter()
                .map(|s| {
                    let id = ps_mac_to_id_str(&s.pmc_mac, &resolved.mac_to_id);
                    rpc::FirmwareUpdateStatus {
                        result: Some(if s.error.is_none() {
                            success_result(&id)
                        } else {
                            error_result(&id, s.error.unwrap_or_default())
                        }),
                        state: map_fw_state(s.state),
                        target_version: s.target_version,
                        updated_at: None,
                    }
                })
                .collect()
        }
        rpc::get_component_firmware_status_request::Target::MachineIds(_) => {
            return Err(Status::unimplemented(
                "machine firmware status is not supported via this RPC",
            ));
        }
    };

    Ok(Response::new(rpc::GetComponentFirmwareStatusResponse {
        statuses,
    }))
}

// ---- List Firmware Versions ----

pub(crate) async fn list_component_firmware_versions(
    api: &Api,
    request: Request<rpc::ListComponentFirmwareVersionsRequest>,
) -> Result<Response<rpc::ListComponentFirmwareVersionsResponse>, Status> {
    log_request_data(&request);
    let cm = require_component_manager(api)?;
    let req = request.into_inner();

    let target = req
        .target
        .ok_or_else(|| Status::invalid_argument("target is required"))?;

    match target {
        rpc::list_component_firmware_versions_request::Target::SwitchIds(_) => {
            let versions = cm
                .nv_switch
                .list_firmware_bundles()
                .await
                .map_err(component_manager_error_to_status)?;
            Ok(Response::new(rpc::ListComponentFirmwareVersionsResponse {
                result: Some(success_result("switches")),
                versions,
            }))
        }
        rpc::list_component_firmware_versions_request::Target::PowerShelfIds(list) => {
            let resolved = resolve_power_shelf_endpoints(api, &list.ids).await?;
            let versions = cm
                .power_shelf
                .list_firmware(&resolved.endpoints)
                .await
                .map_err(component_manager_error_to_status)?;
            Ok(Response::new(rpc::ListComponentFirmwareVersionsResponse {
                result: Some(success_result("power_shelves")),
                versions,
            }))
        }
        rpc::list_component_firmware_versions_request::Target::MachineIds(_) => Err(
            Status::unimplemented("machine firmware versions are not supported via this RPC"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use component_manager::types::FirmwareState;
    use tonic::Code;

    use super::*;

    #[test]
    fn error_to_status_unavailable() {
        let st =
            component_manager_error_to_status(ComponentManagerError::Unavailable("gone".into()));
        assert_eq!(st.code(), Code::Unavailable);
        assert!(st.message().contains("gone"));
    }

    #[test]
    fn error_to_status_not_found() {
        let st =
            component_manager_error_to_status(ComponentManagerError::NotFound("missing".into()));
        assert_eq!(st.code(), Code::NotFound);
    }

    #[test]
    fn error_to_status_invalid_argument() {
        let st =
            component_manager_error_to_status(ComponentManagerError::InvalidArgument("bad".into()));
        assert_eq!(st.code(), Code::InvalidArgument);
    }

    #[test]
    fn error_to_status_internal() {
        let st = component_manager_error_to_status(ComponentManagerError::Internal("oops".into()));
        assert_eq!(st.code(), Code::Internal);
    }

    #[test]
    fn error_to_status_passthrough() {
        let original = Status::permission_denied("nope");
        let st = component_manager_error_to_status(ComponentManagerError::Status(original));
        assert_eq!(st.code(), Code::PermissionDenied);
    }

    #[test]
    fn power_action_on() {
        let action = map_power_action(SystemPowerControl::On as i32).unwrap();
        assert!(matches!(action, PowerAction::On));
    }

    #[test]
    fn power_action_graceful_shutdown() {
        let action = map_power_action(SystemPowerControl::GracefulShutdown as i32).unwrap();
        assert!(matches!(action, PowerAction::GracefulShutdown));
    }

    #[test]
    fn power_action_force_off() {
        let action = map_power_action(SystemPowerControl::ForceOff as i32).unwrap();
        assert!(matches!(action, PowerAction::ForceOff));
    }

    #[test]
    fn power_action_graceful_restart() {
        let action = map_power_action(SystemPowerControl::GracefulRestart as i32).unwrap();
        assert!(matches!(action, PowerAction::GracefulRestart));
    }

    #[test]
    fn power_action_force_restart() {
        let action = map_power_action(SystemPowerControl::ForceRestart as i32).unwrap();
        assert!(matches!(action, PowerAction::ForceRestart));
    }

    #[test]
    fn power_action_ac_powercycle() {
        let action = map_power_action(SystemPowerControl::AcPowercycle as i32).unwrap();
        assert!(matches!(action, PowerAction::AcPowercycle));
    }

    #[test]
    fn power_action_invalid_value() {
        let err = map_power_action(9999).unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
    }

    #[test]
    fn fw_state_round_trip_all_variants() {
        let cases = [
            (
                FirmwareState::Unknown,
                rpc::FirmwareUpdateState::FwStateUnknown as i32,
            ),
            (
                FirmwareState::Queued,
                rpc::FirmwareUpdateState::FwStateQueued as i32,
            ),
            (
                FirmwareState::InProgress,
                rpc::FirmwareUpdateState::FwStateInProgress as i32,
            ),
            (
                FirmwareState::Verifying,
                rpc::FirmwareUpdateState::FwStateVerifying as i32,
            ),
            (
                FirmwareState::Completed,
                rpc::FirmwareUpdateState::FwStateCompleted as i32,
            ),
            (
                FirmwareState::Failed,
                rpc::FirmwareUpdateState::FwStateFailed as i32,
            ),
            (
                FirmwareState::Cancelled,
                rpc::FirmwareUpdateState::FwStateCancelled as i32,
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(map_fw_state(input), expected, "mismatch for {input:?}");
        }
    }

    #[test]
    fn make_result_fields() {
        let r = make_result(
            "sw-1",
            rpc::ComponentManagerStatusCode::Success,
            Some("info".into()),
        );
        assert_eq!(r.component_id, "sw-1");
        assert_eq!(r.status, rpc::ComponentManagerStatusCode::Success as i32);
        assert_eq!(r.error, "info");
    }

    #[test]
    fn success_result_has_no_error() {
        let r = success_result("sw-2");
        assert_eq!(r.status, rpc::ComponentManagerStatusCode::Success as i32);
        assert!(r.error.is_empty());
    }

    #[test]
    fn not_found_result_has_error_message() {
        let r = not_found_result("sw-3");
        assert_eq!(r.status, rpc::ComponentManagerStatusCode::NotFound as i32);
        assert!(r.error.contains("sw-3"));
    }

    #[test]
    fn error_result_has_internal_error_status() {
        let r = error_result("sw-4", "boom".into());
        assert_eq!(
            r.status,
            rpc::ComponentManagerStatusCode::InternalError as i32,
        );
        assert_eq!(r.error, "boom");
    }

    fn test_switch_id() -> SwitchId {
        use carbide_uuid::switch::{SwitchIdSource, SwitchType};
        SwitchId::new(SwitchIdSource::Tpm, [0u8; 32], SwitchType::NvLink)
    }

    fn test_power_shelf_id() -> PowerShelfId {
        use carbide_uuid::power_shelf::{PowerShelfIdSource, PowerShelfType};
        PowerShelfId::new(PowerShelfIdSource::Tpm, [0u8; 32], PowerShelfType::Rack)
    }

    #[test]
    fn switch_mac_to_id_str_found() {
        let mac: MacAddress = "AA:BB:CC:DD:EE:01".parse().unwrap();
        let id = test_switch_id();
        let map = HashMap::from([(mac, id)]);
        assert_eq!(switch_mac_to_id_str(&mac, &map), id.to_string());
    }

    #[test]
    fn switch_mac_to_id_str_not_found_falls_back_to_mac() {
        let mac: MacAddress = "AA:BB:CC:DD:EE:01".parse().unwrap();
        let map = HashMap::new();
        assert_eq!(switch_mac_to_id_str(&mac, &map), mac.to_string());
    }

    #[test]
    fn ps_mac_to_id_str_found() {
        let mac: MacAddress = "AA:BB:CC:DD:EE:02".parse().unwrap();
        let id = test_power_shelf_id();
        let map = HashMap::from([(mac, id)]);
        assert_eq!(ps_mac_to_id_str(&mac, &map), id.to_string());
    }

    #[test]
    fn ps_mac_to_id_str_not_found_falls_back_to_mac() {
        let mac: MacAddress = "AA:BB:CC:DD:EE:02".parse().unwrap();
        let map = HashMap::new();
        assert_eq!(ps_mac_to_id_str(&mac, &map), mac.to_string());
    }
}
