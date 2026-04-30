/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::RackId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "Rack ID to show (leave empty for all)")]
    pub rack: Option<RackId>,
}
