/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod delete;
mod list;
mod show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Display managed switch information")]
    Show(show::Args),
    #[clap(about = "List all managed switches")]
    List(list::Args),
    #[clap(about = "Delete a managed switch")]
    Delete(delete::Args),
}
