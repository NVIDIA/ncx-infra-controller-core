/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use ::rpc::forge::DeleteOperatingSystemRequest;

use super::args::Args;
use crate::operating_system::common::str_to_os_id;
use crate::rpc::ApiClient;

pub async fn delete(opts: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let id = str_to_os_id(&opts.id)?;

    api_client
        .0
        .delete_operating_system(DeleteOperatingSystemRequest { id: Some(id) })
        .await
        .map_err(CarbideCliError::from)?;

    println!("Operating system {} deleted.", opts.id);
    Ok(())
}
