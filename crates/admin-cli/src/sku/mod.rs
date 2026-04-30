/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod assign;
mod bulk_update_metadata;
mod common;
mod create;
mod delete;
mod generate;
mod replace;
pub mod show;
mod show_machines;
mod unassign;
mod update_metadata;
mod verify;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Show SKU information", visible_alias = "s")]
    Show(show::Args),
    #[clap(about = "Show what machines are assigned a SKU")]
    ShowMachines(show_machines::Args),
    #[clap(
        about = "Generate SKU information from an existing machine",
        visible_alias = "g"
    )]
    Generate(generate::Args),
    #[clap(about = "Create SKUs from a file", visible_alias = "c")]
    Create(create::Args),
    #[clap(about = "Delete a SKU", visible_alias = "d")]
    Delete(delete::Args),
    #[clap(about = "Assign a SKU to a machine", visible_alias = "a")]
    Assign(assign::Args),
    #[clap(about = "Unassign a SKU from a machine", visible_alias = "u")]
    Unassign(unassign::Args),
    #[clap(about = "Verify a machine against its SKU", visible_alias = "v")]
    Verify(verify::Args),
    #[clap(about = "Update the metadata of a SKU")]
    UpdateMetadata(update_metadata::Args),
    #[clap(about = "Update multiple SKU's metadata from a file")]
    BulkUpdateMetadata(bulk_update_metadata::Args),
    #[clap(about = "Replace the component list of a SKU")]
    Replace(replace::Args),
}
