/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod common;
mod disable;
mod enable;
mod show;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Enable DPF")]
    Enable(enable::Args),
    #[clap(about = "Disable DPF")]
    Disable(disable::Args),
    #[clap(about = "Check Status of DPF")]
    Show(show::Args),
}
