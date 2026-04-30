/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use ::rpc::forge::DeleteOsImageRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let req: DeleteOsImageRequest = args.try_into()?;
    let id = req.id.clone().expect("id is always set by TryFrom<Args>");
    api_client.0.delete_os_image(req).await?;
    println!("OS image {id} deleted successfully.");
    Ok(())
}
