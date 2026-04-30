/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod add;
pub(crate) mod common;
mod delete;
mod erase;
mod replace_all;
mod show;
mod update;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Show expected switch")]
    Show(show::Args),
    #[clap(about = "Add expected switch")]
    Add(add::Args),
    #[clap(about = "Delete expected switch")]
    Delete(delete::Args),
    #[clap(about = "Update expected switch")]
    Update(update::Args),
    #[clap(about = "Replace all expected switches")]
    ReplaceAll(replace_all::Args),
    #[clap(about = "Erase all expected switches")]
    Erase(erase::Args),
}
