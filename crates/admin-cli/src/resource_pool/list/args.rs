/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

#[derive(Parser, Debug, Default)]
pub struct Args;

impl From<Args> for ::rpc::forge::ListResourcePoolsRequest {
    fn from(_args: Args) -> Self {
        Self {
            auto_assignable: None,
        }
    }
}
