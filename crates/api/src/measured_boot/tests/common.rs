/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! tests/common.rs
//!
//! Shared code by measured boot tests.

use std::str::FromStr;

use carbide_uuid::machine::MachineId;
use measured_boot::machine::CandidateMachine;
use model::hardware_info::HardwareInfo;
use model::machine::{CURRENT_STATE_MODEL_VERSION, ManagedHostState};
use sqlx::PgConnection;

pub fn load_topology_json(path: &str) -> HardwareInfo {
    const TEST_DATA_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/measured_boot/tests/test_data"
    );

    let path = format!("{TEST_DATA_DIR}/{path}");
    let data = std::fs::read(path).unwrap();
    serde_json::from_slice::<HardwareInfo>(&data).unwrap()
}

pub async fn create_test_machine(
    txn: &mut PgConnection,
    machine_id: &str,
    topology: &HardwareInfo,
) -> eyre::Result<CandidateMachine> {
    let machine_id = MachineId::from_str(machine_id)?;
    db::machine::create(
        txn,
        None,
        &machine_id,
        ManagedHostState::Ready,
        None,
        CURRENT_STATE_MODEL_VERSION,
    )
    .await?;
    db::machine_topology::create_or_update(txn, &machine_id, topology).await?;
    let machine = db::measured_boot::machine::from_id(txn, machine_id).await?;
    assert_eq!(machine_id, machine.machine_id);
    Ok(machine)
}
