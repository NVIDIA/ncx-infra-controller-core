/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub(crate) mod common;
mod create;
mod delete;
mod get_version;
pub(crate) mod show;
mod show_instances;
mod update;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Create an extension service")]
    Create(create::Args),
    #[clap(about = "Update an extension service")]
    Update(update::Args),
    #[clap(about = "Delete an extension service")]
    Delete(delete::Args),
    #[clap(about = "Show extension service information")]
    Show(show::Args),
    #[clap(about = "Get extension service version information")]
    GetVersion(get_version::Args),
    #[clap(about = "Show instances using an extension service")]
    ShowInstances(show_instances::Args),
}
