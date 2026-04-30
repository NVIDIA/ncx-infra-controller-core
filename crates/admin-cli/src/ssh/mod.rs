/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod common;
mod copy_bfb;
mod disable_rshim;
mod enable_rshim;
mod get_rshim_status;
mod show_obmc_log;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Show Rshim Status")]
    GetRshimStatus(get_rshim_status::Args),
    #[clap(about = "Disable Rshim")]
    DisableRshim(disable_rshim::Args),
    #[clap(about = "EnableRshim")]
    EnableRshim(enable_rshim::Args),
    #[clap(about = "Copy BFB to the DPU BMC's RSHIM ")]
    CopyBfb(copy_bfb::Args),
    #[clap(about = "Show the DPU's BMC's OBMC log")]
    ShowObmcLog(show_obmc_log::Args),
}
