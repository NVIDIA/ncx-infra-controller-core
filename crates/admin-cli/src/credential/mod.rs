/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod add_bmc;
mod add_dpu_factory_default;
mod add_host_factory_default;
mod add_nmxm;
mod add_uefi;
mod add_ufm;
mod bgp;
mod common;
mod delete_bmc;
mod delete_nmxm;
mod delete_ufm;
mod generate_ufm_cert;

#[cfg(test)]
mod tests;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Clone, Dispatch)]
#[clap(rename_all = "kebab_case")]
pub enum Cmd {
    #[clap(about = "Add UFM credential")]
    AddUFM(add_ufm::Args),
    #[clap(about = "Delete UFM credential")]
    DeleteUFM(delete_ufm::Args),
    #[clap(about = "Generate UFM credential")]
    GenerateUFMCert(generate_ufm_cert::Args),
    #[clap(about = "Add BMC credentials")]
    AddBMC(add_bmc::Args),
    #[clap(about = "Delete BMC credentials")]
    DeleteBMC(delete_bmc::Args),
    #[clap(
        about = "Add site-wide DPU UEFI default credential (NOTE: this parameter can be set only once)"
    )]
    AddUefi(add_uefi::Args),
    #[clap(about = "Add manufacturer factory default BMC user/pass for a given vendor")]
    AddHostFactoryDefault(add_host_factory_default::Args),
    #[clap(about = "Add manufacturer factory default BMC user/pass for the DPUs")]
    AddDpuFactoryDefault(add_dpu_factory_default::Args),
    #[clap(about = "Add NmxM credentials")]
    AddNmxM(add_nmxm::Args),
    #[clap(about = "Delete NmxM credentials")]
    DeleteNmxM(delete_nmxm::Args),
    #[clap(about = "Manage leaf BGP passwords", subcommand)]
    Bgp(bgp::Cmd),
}
