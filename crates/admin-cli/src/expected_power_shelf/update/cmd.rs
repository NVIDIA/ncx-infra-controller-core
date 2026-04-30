/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::ExpectedPowerShelf;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn update(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let request: ExpectedPowerShelf = data.try_into()?;
    api_client.0.update_expected_power_shelf(request).await?;
    Ok(())
}
