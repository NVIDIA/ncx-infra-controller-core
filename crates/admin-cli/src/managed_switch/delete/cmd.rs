/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::SwitchDeletionRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let switch_id = data
        .parse_switch_id()
        .map_err(|e| color_eyre::eyre::eyre!(e))?;
    api_client
        .0
        .delete_switch(SwitchDeletionRequest {
            id: Some(switch_id),
        })
        .await?;
    println!("Switch deleted successfully.");
    Ok(())
}
