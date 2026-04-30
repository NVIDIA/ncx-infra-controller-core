/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[derive(clap::Subcommand, Debug)]
#[clap(rename_all = "kebab-case")]
pub enum Args {
    #[clap(about = "Print network status of all machines")]
    Status,
    #[clap(about = "Machine network configuration, used by VPC.")]
    Config(crate::machine::NetworkConfigQuery),
}
