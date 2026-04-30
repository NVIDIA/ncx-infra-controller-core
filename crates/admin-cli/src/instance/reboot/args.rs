/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::instance::InstanceId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short, long)]
    pub instance: InstanceId,

    #[clap(short, long, action)]
    pub custom_pxe: bool,

    #[clap(short, long, action)]
    pub apply_updates_on_reboot: bool,
}
