/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::power_shelf::PowerShelfId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Power Shelf ID to force delete.")]
    pub power_shelf_id: PowerShelfId,

    #[clap(
        short = 'd',
        long,
        action,
        help = "Delete machine interfaces associated with this power shelf."
    )]
    pub delete_interfaces: bool,
}
