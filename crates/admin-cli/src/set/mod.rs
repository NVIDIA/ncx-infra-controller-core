/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod bmc_proxy;
mod create_machines;
mod log_filter;
mod site_explorer_enabled;
mod tracing_enabled;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Set RUST_LOG")]
    LogFilter(log_filter::Args),
    #[clap(about = "Set create_machines")]
    CreateMachines(create_machines::Args),
    #[clap(about = "Enable or disable site-explorer")]
    SiteExplorer(site_explorer_enabled::Args),
    #[clap(about = "Set bmc_proxy")]
    BmcProxy(bmc_proxy::Args),
    #[clap(
        about = "Configure whether trace/span information is sent to an OTLP endpoint like Tempo"
    )]
    TracingEnabled(tracing_enabled::Args),
}
