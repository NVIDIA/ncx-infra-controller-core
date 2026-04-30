/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod apply;
mod create;
mod delete;
mod get;
mod history;
mod list;
mod set_default;
mod status;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Create a new Rack firmware configuration from JSON file")]
    Create(create::Args),

    #[clap(about = "Get a Rack firmware configuration by ID")]
    Get(get::Args),

    #[clap(about = "List all Rack firmware configurations")]
    List(list::Args),

    #[clap(about = "Delete a Rack firmware configuration")]
    Delete(delete::Args),

    #[clap(about = "Apply firmware to all devices in a rack")]
    Apply(apply::Args),

    #[clap(about = "Check the status of an async firmware update job")]
    Status(status::Args),

    #[clap(about = "Show history of rack firmware apply operations")]
    History(history::Args),

    #[clap(about = "Set a firmware configuration as the default for its hardware type")]
    SetDefault(set_default::Args),
}
