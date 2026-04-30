/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::ExpectedPowerShelfRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let req: ExpectedPowerShelfRequest = data.try_into()?;
    api_client.0.delete_expected_power_shelf(req).await?;
    Ok(())
}
