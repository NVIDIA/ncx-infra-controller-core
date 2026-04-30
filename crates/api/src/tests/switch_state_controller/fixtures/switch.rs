/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::switch::SwitchId;
use model::switch::SwitchControllerState;
use sqlx::PgConnection;

/// Helper function to set switch controller state directly in database
pub async fn set_switch_controller_state(
    txn: &mut PgConnection,
    switch_id: &SwitchId,
    state: SwitchControllerState,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE switches SET controller_state = $1 WHERE id = $2")
        .bind(serde_json::to_value(state).unwrap())
        .bind(switch_id)
        .execute(txn)
        .await?;

    Ok(())
}

/// Helper function to mark switch as deleted
pub async fn mark_switch_as_deleted(
    txn: &mut PgConnection,
    switch_id: &SwitchId,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE switches SET deleted = NOW() WHERE id = $1")
        .bind(switch_id)
        .execute(txn)
        .await?;

    Ok(())
}
