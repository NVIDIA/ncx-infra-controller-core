/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handler for SwitchControllerState::BomValidating.

use carbide_uuid::switch::SwitchId;
use model::switch::{BomValidatingState, Switch, SwitchControllerState};

use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};
use crate::state_controller::switch::context::SwitchStateHandlerContextObjects;

/// Handles the BomValidating state for a switch.
pub async fn handle_bom_validating(
    _switch_id: &SwitchId,
    state: &mut Switch,
    _ctx: &mut StateHandlerContext<'_, SwitchStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<SwitchControllerState>, StateHandlerError> {
    let bom_validating_state = match &state.controller_state.value {
        SwitchControllerState::BomValidating {
            bom_validating_state,
        } => bom_validating_state,
        _ => unreachable!("handle_bom_validating called with non-BomValidating state"),
    };

    match bom_validating_state {
        BomValidatingState::BomValidationComplete => {
            tracing::info!("BOM Validating Switch: BomValidationComplete, moving to Ready");
            Ok(StateHandlerOutcome::transition(
                SwitchControllerState::Ready,
            ))
        }
    }
}
