/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod attach;
mod common;
mod create;
mod delete;
mod detach;
mod show;
mod show_attachments;
mod update;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Create a network security group", visible_alias = "c")]
    Create(create::Args),

    #[clap(
        about = "Show one or more network security groups",
        visible_alias = "s"
    )]
    Show(show::Args),

    #[clap(about = "Delete a network security group", visible_alias = "d")]
    Delete(delete::Args),

    #[clap(about = "Update a network security group", visible_alias = "u")]
    Update(update::Args),

    #[clap(
        about = "Show info about the objects referencing a network security group",
        visible_alias = "a"
    )]
    ShowAttachments(show_attachments::Args),

    #[clap(
        about = "Attach a network security group to a VPC or instance",
        visible_alias = "x"
    )]
    Attach(attach::Args),

    #[clap(
        about = "Remove a network security group from a VPC or instance",
        visible_alias = "r"
    )]
    Detach(detach::Args),
}
