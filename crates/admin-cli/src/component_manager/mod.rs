/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub(crate) mod common;
mod status;
mod update_firmware;
mod versions;

use clap::Parser;

use crate::cfg::dispatch::Dispatch;

#[derive(Parser, Debug, Dispatch)]
pub enum Cmd {
    #[clap(about = "Queue component firmware updates")]
    UpdateFirmware(update_firmware::Args),

    #[clap(
        about = "Get component firmware update status",
        visible_alias = "status"
    )]
    GetFirmwareUpdateStatus(status::Args),

    #[clap(
        about = "List available component firmware versions",
        visible_alias = "versions"
    )]
    GetFirmwareVersions(versions::Args),
}
