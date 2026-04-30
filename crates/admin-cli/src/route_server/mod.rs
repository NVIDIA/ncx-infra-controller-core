/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod add;
mod common;
mod get;
mod remove;
mod replace;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Get all route servers")]
    Get(get::Args),

    #[clap(about = "Add route server addresses")]
    Add(add::Args),

    #[clap(about = "Remove route server addresses")]
    Remove(remove::Args),

    #[clap(about = "Replace all route server addresses")]
    Replace(replace::Args),
}
