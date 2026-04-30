/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::{BmcEndpointRequest, CopyBfbToDpuRshimRequest, SshRequest};

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn copy_bfb_to_dpu_rshim(api_client: &ApiClient, args: Args) -> CarbideCliResult<()> {
    api_client
        .0
        .copy_bfb_to_dpu_rshim(CopyBfbToDpuRshimRequest {
            ssh_request: Some(SshRequest {
                endpoint_request: Some(BmcEndpointRequest {
                    ip_address: args.address.to_string(),
                    mac_address: args.mac.map(|m| m.to_string()),
                }),
            }),
            host_bmc_ip: args.host_bmc_ip,
            pre_copy_powercycle: args.pre_copy_powercycle,
        })
        .await?;

    tracing::info!(
        "BFB recovery triggered for {}. Track progress via: site-explorer get-report endpoint {}",
        args.address,
        args.address
    );
    Ok(())
}
