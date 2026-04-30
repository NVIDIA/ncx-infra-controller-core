/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::switch::SwitchId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    pub switch_id: SwitchId,
    pub report_source: String,
}
