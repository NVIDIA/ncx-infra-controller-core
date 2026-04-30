/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod common;
mod create;
mod delete;
mod show;
mod update;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(
        about = "Create an OS image entry in the OS catalog for a tenant.",
        visible_alias = "c"
    )]
    Create(create::Args),
    #[clap(
        about = "Show one or more OS image entries in the catalog.",
        visible_alias = "s"
    )]
    Show(show::Args),
    #[clap(
        about = "Delete an OS image entry that is not used on any instances.",
        visible_alias = "d"
    )]
    Delete(delete::Args),
    #[clap(
        about = "Update the authentication details or name and description for an OS image.",
        visible_alias = "u"
    )]
    Update(update::Args),
}
