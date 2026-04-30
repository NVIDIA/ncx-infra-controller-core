/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::instance::InstanceId;
use clap::Parser;
use rpc::forge::InstanceOperatingSystemConfig;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short, long, required(true))]
    pub instance: InstanceId,
    #[clap(
        long,
        required(true),
        help = "OS definition in JSON format",
        value_name = "OS_JSON"
    )]
    pub os: InstanceOperatingSystemConfig,
}
