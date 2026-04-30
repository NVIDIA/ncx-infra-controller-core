/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::RackId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "Rack ID to get profile for")]
    pub rack_id: RackId,
}
