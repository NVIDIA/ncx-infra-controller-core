/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[clap(group(
        ArgGroup::new("release_instance")
        .required(true)
        .args(&["instance", "machine", "label_key"])))]
pub struct Args {
    #[clap(short, long)]
    pub instance: Option<String>,

    #[clap(short, long)]
    pub machine: Option<MachineId>,

    #[clap(long, help = "The key of label instance to query")]
    pub label_key: Option<String>,

    #[clap(long, help = "The value of label instance to query")]
    pub label_value: Option<String>,
}
