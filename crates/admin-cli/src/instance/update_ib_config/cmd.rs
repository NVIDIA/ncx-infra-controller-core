/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};

use super::args::Args;
use crate::instance::common::GlobalOptions;
use crate::rpc::ApiClient;

pub async fn update_ib_config(
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
                config.infiniband = Some(update_request.config);
            },
            |_metadata| {},
            opts.cloud_unsafe_op,
        )
        .await
    {
        Ok(i) => {
            tracing::info!("update-ib-config was successful. Updated instance: {:?}", i);
        }
        Err(e) => {
            tracing::info!("update-ib-config failed with {} ", e);
        }
    };
    Ok(())
}
