/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod create;
mod delete;
mod show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Display logical partition information")]
    Show(show::Args),
    #[clap(about = "Create logical partition")]
    Create(create::Args),
    #[clap(about = "Delete logical partition")]
    Delete(delete::Args),
}
