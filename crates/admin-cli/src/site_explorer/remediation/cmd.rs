/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use ::rpc::forge::PauseExploredEndpointRemediationRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn remediation(api_client: &ApiClient, opts: Args) -> CarbideCliResult<()> {
    if opts.pause {
        api_client
            .0
            .pause_explored_endpoint_remediation(PauseExploredEndpointRemediationRequest {
                ip_address: opts.address.clone(),
                pause: true,
            })
            .await?;
        println!("Remediation paused for endpoint {}", opts.address);
    } else if opts.resume {
        api_client
            .0
            .pause_explored_endpoint_remediation(PauseExploredEndpointRemediationRequest {
                ip_address: opts.address.clone(),
                pause: false,
            })
            .await?;
        println!("Remediation resumed for endpoint {}", opts.address);
    } else {
        return Err(CarbideCliError::RequireOneError("--pause", "--resume"));
    }
    Ok(())
}
