/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use color_eyre::Result;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn delete_rack(api_client: &ApiClient, delete_opts: Args) -> Result<()> {
    api_client.0.delete_rack(delete_opts).await?;
    Ok(())
}
