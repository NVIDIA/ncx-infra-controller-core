/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::power_shelf::PowerShelfId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    pub power_shelf_id: PowerShelfId,
    pub report_source: String,
}
