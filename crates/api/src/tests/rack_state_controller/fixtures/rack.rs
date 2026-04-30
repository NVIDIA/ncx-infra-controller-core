/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::rack::RackId;
use model::rack::RackState;
use sqlx::PgConnection;

/// Helper function to set rack controller state directly in database
pub async fn set_rack_controller_state(
    txn: &mut PgConnection,
    rack_id: &RackId,
    state: RackState,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE racks SET controller_state = $1 WHERE id = $2")
        .bind(serde_json::to_value(state).unwrap())
        .bind(rack_id)
        .execute(txn)
        .await?;

    Ok(())
}
