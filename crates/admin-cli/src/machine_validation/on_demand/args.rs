/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Args {
    #[clap(about = "Start on demand machine validation")]
    Start(OnDemandOptions),
}

#[derive(Parser, Debug)]
#[clap(disable_help_flag = true)]
pub struct OnDemandOptions {
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,

    #[clap(short, long, help = "Machine id for start validation")]
    pub machine: MachineId,

    #[clap(long, help = "Results history")]
    pub tags: Option<Vec<String>>,

    #[clap(long, help = "Allowed tests")]
    pub allowed_tests: Option<Vec<String>>,

    #[clap(long, default_value = "false", help = "Run not verfified tests")]
    pub run_unverfied_tests: bool,

    #[clap(long, help = "Contexts")]
    pub contexts: Option<Vec<String>>,
}
