/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod allocate;
pub(crate) mod common;
mod reboot;
mod release;
mod show;
mod update_ib_config;
mod update_nvlink_config;
mod update_os;

// Cross-module re-exports for jump module
// Cross-module re-export for rpc module
pub use allocate::args::Args as AllocateInstance;
pub use show::args::Args as ShowInstance;
pub use show::cmd::handle_show;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Display instance information")]
    Show(show::Args),
    #[clap(about = "Reboot instance, potentially applying firmware updates")]
    Reboot(reboot::Args),
    #[clap(about = "De-allocate instance")]
    Release(release::Args),
    #[clap(about = "Allocate instance")]
    Allocate(allocate::Args),
    #[clap(about = "Update instance OS")]
    UpdateOS(update_os::Args),
    #[clap(about = "Update instance IB configuration")]
    UpdateIbConfig(update_ib_config::Args),
    #[clap(about = "Update instance NVLink configuration")]
    UpdateNvLinkConfig(update_nvlink_config::Args),
}
