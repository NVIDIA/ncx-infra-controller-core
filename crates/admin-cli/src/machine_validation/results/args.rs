/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
pub enum Args {
    #[clap(about = "Show results")]
    Show(ShowResultsOptions),
}

#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("group").required(true).multiple(true).args(&[
    "validation_id",
    "test_name",
    "machine",
    ])))]
pub struct ShowResultsOptions {
    #[clap(
        short = 'm',
        long,
        group = "group",
        help = "Show machine validation result of a machine"
    )]
    pub machine: Option<MachineId>,

    #[clap(short = 'v', long, group = "group", help = "Machine validation id")]
    pub validation_id: Option<String>,

    #[clap(
        short = 't',
        long,
        group = "group",
        requires("validation_id"),
        help = "Name of the test case"
    )]
    pub test_name: Option<String>,

    #[clap(long, default_value = "false", help = "Results history")]
    pub history: bool,
}
