/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::ExpectedSwitchRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let req: ExpectedSwitchRequest = data.try_into()?;
    api_client.0.delete_expected_switch(req).await?;
    Ok(())
}
