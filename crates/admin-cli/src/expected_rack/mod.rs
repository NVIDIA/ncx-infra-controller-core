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

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Show expected rack")]
    Show(show::Args),
    #[clap(about = "Add expected rack")]
    Add(add::Args),
    #[clap(about = "Delete expected rack")]
    Delete(delete::Args),
    #[clap(about = "Update expected rack")]
    Update(update::Args),
    #[clap(about = "Replace all expected racks")]
    ReplaceAll(replace_all::Args),
    #[clap(about = "Erase all expected racks")]
    Erase(erase::Args),
}
