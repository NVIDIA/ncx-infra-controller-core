/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod bash_command;

#[async_trait::async_trait]
pub trait Command: std::fmt::Debug + Send + Sync {
    async fn run(&mut self) -> eyre::Result<String>;
}
