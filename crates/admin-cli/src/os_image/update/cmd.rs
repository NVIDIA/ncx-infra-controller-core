/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::{Args, UpdateRequest};
use crate::rpc::ApiClient;

pub async fn update(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let req: UpdateRequest = args.try_into()?;
    let image = api_client
        .update_os_image(
            req.id,
            req.auth_type,
            req.auth_token,
            req.name,
            req.description,
        )
        .await?;
    if let Some(x) = image.attributes {
        if let Some(y) = x.id {
            println!("OS image {y} updated successfully.");
        } else {
            eprintln!("Updating the OS image may have failed, image id missing.");
        }
    } else {
        eprintln!("Updating the OS image may have failed, image attributes missing.");
    }
    Ok(())
}
