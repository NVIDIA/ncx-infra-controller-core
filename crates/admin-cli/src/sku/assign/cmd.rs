/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use carbide_uuid::machine::MachineId;
use rpc::forge::SkuMachinePair;

use crate::rpc::ApiClient;

pub async fn assign(
    sku_id: String,
    machine_id: MachineId,
    force: bool,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    api_client
        .0
        .assign_sku_to_machine(SkuMachinePair {
            sku_id,
            machine_id: Some(machine_id),
            force,
        })
        .await?;
    Ok(())
}
