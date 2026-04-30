/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliResult, OutputFormat};
use carbide_uuid::machine::MachineId;

use crate::dpf::common::DpfQuery;
use crate::rpc::ApiClient;

pub async fn modify_dpf_state(
    query: &DpfQuery,
    _format: OutputFormat, // TODO: Implement json output handling.
    api_client: &ApiClient,
    enabled: bool,
) -> CarbideCliResult<()> {
    let host: MachineId = query.try_into()?;
    api_client.modify_dpf_state(host, enabled).await?;
    println!("DPF state modified for machine {host} with state {enabled} successfully!!",);
    Ok(())
}
