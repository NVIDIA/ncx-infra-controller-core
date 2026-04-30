/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod clear_error;
mod common;
mod copy_bfb_to_dpu_rshim;
mod delete;
mod explore;
pub mod get_report;
mod have_credentials;
mod is_bmc_in_managed_host;
mod re_explore;
mod remediation;

#[cfg(test)]
mod tests;

use clap::Parser;
// Re-export for cross-module use by jump/cmds.rs
pub use get_report::args::{Args as GetReportMode, EndpointInfo};
pub use get_report::cmd::show_discovered_managed_host as show_site_explorer_discovered_managed_host;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Retrieves the latest site exploration report", subcommand)]
    GetReport(get_report::Args),
    #[clap(
        about = "Asks carbide-api to explore a single host and prints the report. Does not store it."
    )]
    Explore(explore::Args),
    #[clap(
        about = "Asks carbide-api to explore a single host in the next exploration cycle. The results will be stored."
    )]
    ReExplore(re_explore::Args),
    #[clap(about = "Clear the last known error for the BMC in the latest site exploration report.")]
    ClearError(clear_error::Args),
    #[clap(about = "Delete an explored endpoint from the database.")]
    Delete(delete::Args),
    #[clap(about = "Control remediation actions for an explored endpoint.")]
    Remediation(remediation::Args),
    IsBmcInManagedHost(is_bmc_in_managed_host::Args),
    HaveCredentials(have_credentials::Args),
    CopyBfbToDpuRshim(copy_bfb_to_dpu_rshim::Args),
}
