/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fs;

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use ::rpc::forge as forgerpc;
use carbide_uuid::machine::MachineId;

use super::args::MachineHardwareInfoGpus;
use crate::rpc::ApiClient;

pub async fn handle_update_machine_hardware_info_gpus(
    api_client: &ApiClient,
    gpus: MachineHardwareInfoGpus,
) -> CarbideCliResult<()> {
    let gpu_file_contents = fs::read_to_string(gpus.gpu_json_file)?;
    let gpus_from_json: Vec<::rpc::machine_discovery::Gpu> =
        serde_json::from_str(&gpu_file_contents)?;
    api_client
        .update_machine_hardware_info(
            gpus.machine,
            forgerpc::MachineHardwareInfoUpdateType::Gpus,
            gpus_from_json,
        )
        .await
}

pub fn handle_show_machine_hardware_info(
    _api_client: &ApiClient,
    _output_file: &mut Box<dyn tokio::io::AsyncWrite + Unpin>,
    _output_format: &OutputFormat,
    _machine_id: MachineId,
) -> CarbideCliResult<()> {
    Err(CarbideCliError::NotImplemented(
        "machine hardware output".to_string(),
    ))
}
