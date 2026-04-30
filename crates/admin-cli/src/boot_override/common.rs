/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineInterfaceId;
use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct BootOverride {
    pub interface_id: MachineInterfaceId,
}
