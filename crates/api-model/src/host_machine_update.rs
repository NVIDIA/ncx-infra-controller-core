/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct HostMachineUpdate {
    pub id: MachineId,
}
