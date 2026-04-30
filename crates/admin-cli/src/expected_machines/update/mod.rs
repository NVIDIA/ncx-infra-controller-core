/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod args;
pub mod cmd;

use std::path::Path;

use ::rpc::admin_cli::CarbideCliResult;
pub use args::Args;

use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;
use crate::expected_machines::common::ExpectedMachineJson;

/// `expected-machine update <file>`: deserializes `ExpectedMachineJson` and calls
/// `patch_expected_machine` with every field from the file (full replacement style), including
/// optional `bmc_ip_address` when present in JSON.
impl Run for Args {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        let json_file_path = Path::new(&self.filename);
        let file_content = std::fs::read_to_string(json_file_path)?;
        let expected_machine: ExpectedMachineJson = serde_json::from_str(&file_content)?;

        let metadata = expected_machine.metadata.unwrap_or_default();

        // Patch merges with the server record; we pass all fields from JSON so the result matches the file.
        ctx.api_client
            .patch_expected_machine(
                Some(expected_machine.bmc_mac_address),
                None,
                Some(expected_machine.bmc_username),
                Some(expected_machine.bmc_password),
                Some(expected_machine.chassis_serial_number),
                expected_machine.fallback_dpu_serial_numbers,
                Some(metadata.name),
                Some(metadata.description),
                Some(
                    metadata
                        .labels
                        .into_iter()
                        .map(|label| {
                            if let Some(value) = label.value {
                                format!("{}:{}", label.key, value)
                            } else {
                                label.key
                            }
                        })
                        .collect(),
                ),
                expected_machine.sku_id,
                expected_machine.rack_id,
                expected_machine.default_pause_ingestion_and_poweron,
                expected_machine.dpf_enabled,
                expected_machine.bmc_ip_address,
                expected_machine.bmc_retain_credentials,
            )
            .await?;
        Ok(())
    }
}
