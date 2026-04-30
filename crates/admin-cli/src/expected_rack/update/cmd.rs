/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::ExpectedRack;

use super::args::Args;
use crate::rpc::ApiClient;

/// update updates an existing expected rack's rack_profile_id and metadata.
pub async fn update(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let req: ExpectedRack = data.try_into()?;
    api_client.0.update_expected_rack(req).await?;
    Ok(())
}
