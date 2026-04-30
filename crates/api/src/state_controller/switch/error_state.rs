/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handler for SwitchControllerState::Error.

use carbide_uuid::switch::SwitchId;
use model::switch::{Switch, SwitchControllerState};

use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};
use crate::state_controller::switch::context::SwitchStateHandlerContextObjects;

/// Handles the Error state for a switch.
/// If marked for deletion, transition to Deleting; otherwise wait for manual intervention.
pub async fn handle_error(
    _switch_id: &SwitchId,
    state: &mut Switch,
    _ctx: &mut StateHandlerContext<'_, SwitchStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<SwitchControllerState>, StateHandlerError> {
    tracing::info!("Switch is in error state {}", _switch_id.to_string());
    if state.is_marked_as_deleted() {
        Ok(StateHandlerOutcome::transition(
            SwitchControllerState::Deleting,
        ))
    } else {
        Ok(StateHandlerOutcome::do_nothing())
    }
}
