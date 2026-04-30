/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};

use super::args::Args;
use crate::cfg::runtime::RuntimeContext;

pub async fn handle_delete(args: Args, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
    if !ctx.config.cloud_unsafe_op_enabled {
        return Err(CarbideCliError::GenericError(
            "Operation not allowed due to potential inconsistencies with cloud database."
                .to_owned(),
        ));
    }
    ctx.api_client.0.delete_network_segment(args).await?;
    Ok(())
}
