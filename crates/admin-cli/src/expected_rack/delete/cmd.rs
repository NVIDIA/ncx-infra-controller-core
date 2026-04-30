/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::Args;
use crate::rpc::ApiClient;

/// delete deletes an expected rack by its rack_id.
pub async fn delete(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    api_client.0.delete_expected_rack(data).await?;
    Ok(())
}
