/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod add;
mod add_bulk;
mod delete;
mod show;
mod show_unmatched_ek;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Show all TPM CA certificates")]
    Show(show::Args),
    #[clap(about = "Delete TPM CA certificate with a given id")]
    Delete(delete::Args),
    #[clap(about = "Add TPM CA certificate encoded in DER/CER/PEM format in a given file")]
    Add(add::Args),
    #[clap(about = "Show TPM EK certificates for which there is no CA match")]
    ShowUnmatchedEk(show_unmatched_ek::Args),
    #[clap(about = "Add all certificates in a dir as CA certificates")]
    AddBulk(add_bulk::Args),
}
