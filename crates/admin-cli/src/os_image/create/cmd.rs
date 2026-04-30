/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn create(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let image_attrs: ::rpc::forge::OsImageAttributes = args.try_into()?;
    let image = api_client.0.create_os_image(image_attrs).await?;
    if let Some(x) = image.attributes {
        if let Some(y) = x.id {
            println!("OS image {y} created successfully.");
        } else {
            eprintln!("OS image creation may have failed, image id missing.");
        }
    } else {
        eprintln!("OS image creation may have failed, image attributes missing.");
    }
    Ok(())
}
