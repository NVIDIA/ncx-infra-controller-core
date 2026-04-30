/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use carbide_uuid::machine::MachineId;

use crate::rpc::ApiClient;

pub async fn verify(machine_id: MachineId, api_client: &ApiClient) -> CarbideCliResult<()> {
    api_client.0.verify_sku_for_machine(machine_id).await?;
    Ok(())
}
