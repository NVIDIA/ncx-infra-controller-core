/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod args;
pub mod cmd;

use ::rpc::admin_cli::CarbideCliResult;
pub use args::Args;

use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;

/// `expected-machine patch`: forwards CLI flags to `ApiClient::patch_expected_machine` (partial
/// update; unset flags keep existing values). `--bmc-ip-address` uses the same server-side
/// static-interface logic as a full RPC update.
impl Run for Args {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        if let Err(e) = self.validate() {
            eprintln!("{e}");
            return Ok(());
        }
        ctx.api_client
            .patch_expected_machine(
                self.bmc_mac_address,
                self.id.map(|id| id.to_string()),
                self.bmc_username,
                self.bmc_password,
                self.chassis_serial_number,
                self.fallback_dpu_serial_numbers,
                self.meta_name,
                self.meta_description,
                self.labels,
                self.sku_id,
                self.rack_id,
                self.default_pause_ingestion_and_poweron,
                self.dpf_enabled,
                self.bmc_ip_address,
                self.bmc_retain_credentials,
            )
            .await?;
        Ok(())
    }
}
