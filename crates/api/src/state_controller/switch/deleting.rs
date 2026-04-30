/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handler for SwitchControllerState::Deleting.

use carbide_uuid::switch::SwitchId;
use db::switch as db_switch;
use model::switch::{Switch, SwitchControllerState};

use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};
use crate::state_controller::switch::context::SwitchStateHandlerContextObjects;

/// Handles the Deleting state for a switch.
/// TODO: Implement full deletion logic (check in use, shut down, release resources).
pub async fn handle_deleting(
    switch_id: &SwitchId,
    _state: &mut Switch,
    ctx: &mut StateHandlerContext<'_, SwitchStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<SwitchControllerState>, StateHandlerError> {
    tracing::info!("Deleting Switch {}", switch_id.to_string());
    let mut txn = ctx.services.db_pool.begin().await?;
    db_switch::final_delete(*switch_id, &mut txn).await?;
    Ok(StateHandlerOutcome::deleted().with_txn(txn))
}
