/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};

pub fn str_to_rpc_uuid(id: &str) -> CarbideCliResult<::rpc::common::Uuid> {
    let id: ::rpc::common::Uuid = uuid::Uuid::parse_str(id)
        .map_err(|e| CarbideCliError::GenericError(e.to_string()))?
        .into();
    Ok(id)
}
