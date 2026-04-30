/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::dpa_interface::DpaInterfaceId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The DPA Interface ID to query, leave empty for all (default)")]
    pub id: Option<DpaInterfaceId>,
}
