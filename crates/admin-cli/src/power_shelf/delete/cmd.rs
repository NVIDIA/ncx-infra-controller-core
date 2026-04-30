/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::PowerShelfDeletionRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let power_shelf_id = data
        .parse_power_shelf_id()
        .map_err(|e| color_eyre::eyre::eyre!(e))?;
    api_client
        .0
        .delete_power_shelf(PowerShelfDeletionRequest {
            id: Some(power_shelf_id),
        })
        .await?;
    println!("Power shelf deleted successfully.");
    Ok(())
}
