/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn add(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    api_client.0.add_expected_rack(data).await?;
    Ok(())
}
