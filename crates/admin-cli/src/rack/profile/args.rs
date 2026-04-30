/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use super::show;

#[derive(Parser, Debug, Clone)]
pub enum Args {
    #[clap(about = "Show rack profile for a given rack")]
    Show(show::Args),
}
