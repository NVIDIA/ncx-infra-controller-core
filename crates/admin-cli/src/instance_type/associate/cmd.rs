/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn create_association(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let req: ::rpc::forge::AssociateMachinesWithInstanceTypeRequest = args.try_into()?;
    api_client
        .0
        .associate_machines_with_instance_type(req)
        .await?;

    println!("Association is created successfully!!");

    Ok(())
}
