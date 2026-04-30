/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use carbide_uuid::power_shelf::PowerShelfId;
use carbide_uuid::switch::SwitchId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "association_type")]
pub enum InterfaceAssociationType {
    None = 0,
    Machine = 1,
    Switch = 2,
    PowerShelf = 3,
}

pub enum MachineInterfaceAssociation {
    Machine(MachineId),
    Switch(SwitchId),
    PowerShelf(PowerShelfId),
}
