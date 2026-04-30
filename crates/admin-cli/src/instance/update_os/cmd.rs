/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};

use super::args::Args;
use crate::instance::common::GlobalOptions;
use crate::rpc::ApiClient;

pub async fn update_os(
    api_client: &ApiClient,
    update_request: Args,
    opts: GlobalOptions<'_>,
) -> CarbideCliResult<()> {
    if opts.cloud_unsafe_op.is_none() {
        return Err(CarbideCliError::GenericError(
            "Operation not allowed due to potential inconsistencies with cloud database."
                .to_owned(),
        ));
    }

    match api_client
        .update_instance_config_with(
            update_request.instance,
            |config| {
                config.os = Some(update_request.os);
            },
            |_metadata| {},
            opts.cloud_unsafe_op,
        )
        .await
    {
        Ok(i) => {
            tracing::info!("update-os was successful. Updated instance: {:?}", i);
        }
        Err(e) => {
            tracing::info!("update-os failed with {} ", e);
        }
    };
    Ok(())
}
