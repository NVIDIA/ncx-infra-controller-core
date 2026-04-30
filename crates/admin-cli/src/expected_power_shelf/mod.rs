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
    #[clap(about = "Show expected power shelf")]
    Show(show::Args),
    #[clap(about = "Add expected power shelf")]
    Add(add::Args),
    #[clap(about = "Delete expected power shelf")]
    Delete(delete::Args),
    #[clap(about = "Update expected power shelf")]
    Update(update::Args),
    #[clap(about = "Replace all expected power shelves")]
    ReplaceAll(replace_all::Args),
    #[clap(about = "Erase all expected power shelves")]
    Erase(erase::Args),
}
