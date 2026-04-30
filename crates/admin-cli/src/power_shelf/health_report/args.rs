/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

use super::{add, print_empty_template, remove, show};
use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Args {
    #[clap(about = "List health report sources for a power shelf")]
    Show(show::Args),
    #[clap(about = "Insert a health report source for a power shelf")]
    Add(add::Args),
    #[clap(about = "Print an empty health report template")]
    PrintEmptyTemplate(print_empty_template::Args),
    #[clap(about = "Remove a health report source from a power shelf")]
    Remove(remove::Args),
}
