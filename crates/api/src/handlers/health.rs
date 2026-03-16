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

use ::rpc::forge::hardware_machine_leaks::PowerStatus;
use ::rpc::forge::{self as rpc, HealthReportOverride};
use carbide_uuid::machine::MachineId;
use health_report::OverrideMode;
use model::machine::machine_search_config::MachineSearchConfig;
use sqlx::PgConnection;
use tonic::{Request, Response, Status};

use crate::CarbideError;
use crate::api::Api;
use crate::auth::AuthContext;
use crate::handlers::utils::convert_and_log_machine_id;

const TRAY_LEAK_DETECTION_SOURCE: &str = "hardware-health.tray-leak-detection";

// Return a list of machines that have leaks and their associated alerts.
// If the machines list in the request is empty, return a report for all machines.
// Otherwise, confine the leaks report to the machines in the request.
pub async fn get_hardware_leaks_report(
    api: &Api,
    request: Request<rpc::HardwareLeaksReportRequest>,
) -> Result<Response<rpc::HardwareLeaksReportResponse>, Status> {
    let mut txn = api.txn_begin().await?;

    let machine_ids = request.into_inner().machine_ids;

    let machines_with_leaks = db::machine::find_machines_with_leaks(&mut txn, &machine_ids).await?;

    txn.commit().await?;

    let leak_reports = machines_with_leaks
        .into_iter()
        .filter_map(|(machine_id, power_state, overrides)| {
            let report = overrides?.merges.remove(TRAY_LEAK_DETECTION_SOURCE)?;
            let power_status = power_state.and_then(|s| match s {
                model::power_manager::PowerState::On => Some(PowerStatus::On as i32),
                model::power_manager::PowerState::Off => Some(PowerStatus::Off as i32),
                model::power_manager::PowerState::PowerManagerDisabled => None,
            });
            Some(rpc::HardwareMachineLeaks {
                machine_id: Some(machine_id),
                power_status,
                overrides: Some(rpc::HealthReportOverride {
                    report: Some(report.into()),
                    mode: rpc::OverrideMode::Merge as i32,
                }),
            })
        })
        .collect();

    Ok(Response::new(rpc::HardwareLeaksReportResponse {
        leak_reports,
    }))
}

pub async fn list_health_report_overrides(
    api: &Api,
    machine_id: Request<MachineId>,
) -> Result<Response<rpc::ListHealthReportOverrideResponse>, Status> {
    let mut txn = api.txn_begin().await?;

    let machine_id = convert_and_log_machine_id(Some(&machine_id.into_inner()))?;

    let host_machine = db::machine::find_one(&mut txn, &machine_id, MachineSearchConfig::default())
        .await?
        .ok_or_else(|| CarbideError::NotFoundError {
            kind: "machine",
            id: machine_id.to_string(),
        })?;

    txn.commit().await?;

    let ovr = host_machine
        .health_report_overrides
        .clone()
        .into_iter()
        .map(|o| HealthReportOverride {
            report: Some(o.0.into()),
            mode: o.1 as i32,
        })
        .collect();

    Ok(Response::new(rpc::ListHealthReportOverrideResponse {
        overrides: ovr,
    }))
}

async fn remove_by_source(
    txn: &mut PgConnection,
    machine_id: MachineId,
    source: String,
) -> Result<(), CarbideError> {
    let host_machine = db::machine::find_one(
        txn,
        &machine_id,
        MachineSearchConfig {
            // Technically,  an update is going to happen,
            // but we don't seem to need coordination/locking.
            for_update: false,
            ..Default::default()
        },
    )
    .await?
    .ok_or_else(|| CarbideError::NotFoundError {
        kind: "machine",
        id: machine_id.to_string(),
    })?;

    // Ensure this source already exists in override list
    let mode = if host_machine
        .health_report_overrides
        .replace
        .as_ref()
        .map(|o| &o.source)
        == Some(&source)
    {
        OverrideMode::Replace
    } else if host_machine
        .health_report_overrides
        .merges
        .contains_key(&source)
    {
        OverrideMode::Merge
    } else {
        return Err(CarbideError::NotFoundError {
            kind: "machine with source",
            id: source.to_string(),
        });
    };

    db::machine::remove_health_report_override(txn, &machine_id, mode, &source).await?;

    Ok(())
}

pub async fn insert_health_report_override(
    api: &Api,
    request: Request<rpc::InsertHealthReportOverrideRequest>,
) -> Result<Response<()>, Status> {
    let triggered_by = request
        .extensions()
        .get::<AuthContext>()
        .and_then(|ctx| ctx.get_external_user_name())
        .map(String::from);

    let rpc::InsertHealthReportOverrideRequest {
        machine_id,
        r#override: Some(rpc::HealthReportOverride { report, mode }),
    } = request.into_inner()
    else {
        return Err(CarbideError::MissingArgument("override").into());
    };
    let machine_id = convert_and_log_machine_id(machine_id.as_ref())?;
    let Some(report) = report else {
        return Err(CarbideError::MissingArgument("report").into());
    };
    let Ok(mode) = rpc::OverrideMode::try_from(mode) else {
        return Err(CarbideError::InvalidArgument("mode".to_string()).into());
    };
    let mode: OverrideMode = mode.into();
    if machine_id.machine_type().is_dpu() && mode == OverrideMode::Replace {
        return Err(CarbideError::InvalidArgument(
            "DPU's cannot have OverrideMode::Replace health report overrides".to_string(),
        )
        .into());
    }

    let mut txn = api.txn_begin().await?;

    let mut report = health_report::HealthReport::try_from(report.clone())
        .map_err(|e| CarbideError::internal(e.to_string()))?;
    if report.observed_at.is_none() {
        report.observed_at = Some(chrono::Utc::now());
    }
    report.triggered_by = triggered_by;
    report.update_in_alert_since(None);

    // In case a report with the same source exists, either as merge or replace,
    // remove it. If such a report does not exist, ignore error.
    match remove_by_source(&mut txn, machine_id, report.source.clone()).await {
        Ok(_) | Err(CarbideError::NotFoundError { .. }) => {}
        Err(e) => return Err(e.into()),
    }

    db::machine::insert_health_report_override(&mut txn, &machine_id, mode, &report, false).await?;

    txn.commit().await?;

    Ok(Response::new(()))
}

pub async fn remove_health_report_override(
    api: &Api,
    request: Request<rpc::RemoveHealthReportOverrideRequest>,
) -> Result<Response<()>, Status> {
    let mut txn = api.txn_begin().await?;

    let rpc::RemoveHealthReportOverrideRequest { machine_id, source } = request.into_inner();
    let machine_id = convert_and_log_machine_id(machine_id.as_ref())?;
    remove_by_source(&mut txn, machine_id, source).await?;
    txn.commit().await?;

    Ok(Response::new(()))
}
