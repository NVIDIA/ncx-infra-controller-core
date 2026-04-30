/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod common;
mod create;
mod delete;
mod get_artifacts;
mod set_cached_url;
mod show;
mod update;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(
        about = "Show operating system definitions (all, or one by ID).",
        visible_alias = "s"
    )]
    Show(show::Args),
    #[clap(
        about = "Create a new operating system definition.",
        visible_alias = "c"
    )]
    Create(create::Args),
    #[clap(
        about = "Update an existing operating system definition.",
        visible_alias = "u"
    )]
    Update(update::Args),
    #[clap(about = "Delete an operating system definition.", visible_alias = "d")]
    Delete(delete::Args),
    #[clap(
        about = "Get the artifact list for an OS definition.",
        visible_alias = "ga"
    )]
    GetArtifacts(get_artifacts::Args),
    #[clap(
        about = "Set or clear cached_url on OS artifacts.",
        visible_alias = "scu"
    )]
    SetCachedUrl(set_cached_url::Args),
}
