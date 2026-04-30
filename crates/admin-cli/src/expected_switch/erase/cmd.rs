/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn erase(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    if !data.confirm {
        eprintln!("Please set --confirm to confirm you want to erase all expected switches.");
        return Ok(());
    }
    api_client.0.delete_all_expected_switches().await?;
    Ok(())
}
