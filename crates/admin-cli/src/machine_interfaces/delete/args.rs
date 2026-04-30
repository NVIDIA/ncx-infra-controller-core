/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineInterfaceId;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(help = "The interface ID to delete. Redeploy kea after deleting machine interfaces.")]
    pub interface_id: MachineInterfaceId,
}
