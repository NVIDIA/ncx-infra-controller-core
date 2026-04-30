/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug, PartialEq)]
pub enum Args {
    #[clap(about = "Get everything in Json")]
    All,
    #[clap(about = "Get discovered host details.")]
    ManagedHost(ManagedHostInfo),
    #[clap(about = "Get Endpoint details.")]
    Endpoint(EndpointInfo),
}

#[derive(Parser, Debug, PartialEq)]
#[clap(group(ArgGroup::new("selector").required(false).args(&["erroronly", "successonly"])))]
pub struct EndpointInfo {
    #[clap(help = "BMC IP address of Endpoint.")]
    pub address: Option<String>,

    #[clap(
        short,
        long,
        help = "Filter based on vendor. Valid only for table view."
    )]
    pub vendor: Option<String>,

    #[clap(
        long,
        action,
        help = "By default shows all endpoints. If wants to see unpairedonly, choose this option."
    )]
    pub unpairedonly: bool,

    #[clap(long, action, help = "Show only endpoints which have error.")]
    pub erroronly: bool,

    #[clap(long, action, help = "Show only endpoints which have no error.")]
    pub successonly: bool,
}

#[derive(Parser, Debug, PartialEq)]
pub struct ManagedHostInfo {
    #[clap(help = "BMC IP address of host or DPU")]
    pub address: Option<String>,

    #[clap(
        short,
        long,
        help = "Filter based on vendor. Valid only for table view."
    )]
    pub vendor: Option<String>,
}
