/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::{Args, QuarantineOff, QuarantineOn};
use crate::rpc::ApiClient;

pub async fn quarantine_on(api_client: &ApiClient, args: QuarantineOn) -> CarbideCliResult<()> {
    let host = args.host;
    let prior_state = api_client.0.set_managed_host_quarantine_state(args).await?;
    println!(
        "quarantine set for host {}, prior state: {:?}",
        host, prior_state.prior_quarantine_state
    );
    Ok(())
}

pub async fn quarantine_off(api_client: &ApiClient, args: QuarantineOff) -> CarbideCliResult<()> {
    let host = args.host;
    let prior_state = api_client
        .0
        .clear_managed_host_quarantine_state(args)
        .await?;
    println!(
        "quarantine set for host {}, prior state: {:?}",
        host, prior_state.prior_quarantine_state
    );
    Ok(())
}

pub async fn quarantine(api_client: &ApiClient, action: Args) -> CarbideCliResult<()> {
    match action {
        Args::On(args) => quarantine_on(api_client, args).await,
        Args::Off(args) => quarantine_off(api_client, args).await,
    }
}
