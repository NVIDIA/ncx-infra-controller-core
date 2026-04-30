/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handler for SwitchControllerState::Validating.

use carbide_uuid::switch::SwitchId;
use model::switch::{BomValidatingState, Switch, SwitchControllerState, ValidatingState};

use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};
use crate::state_controller::switch::context::SwitchStateHandlerContextObjects;

/// Handles the Validating state for a switch.
/// TODO: Implement Switch validation logic.
pub async fn handle_validating(
    switch_id: &SwitchId,
    state: &mut Switch,
    _ctx: &mut StateHandlerContext<'_, SwitchStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<SwitchControllerState>, StateHandlerError> {
    tracing::info!("Validating Switch {:?}", switch_id);
    let validating_state = match &state.controller_state.value {
        SwitchControllerState::Validating { validating_state } => validating_state,
        _ => unreachable!("handle_validating called with non-Validating state"),
    };

    match validating_state {
        ValidatingState::ValidationComplete => {
            tracing::info!("Validating Switch: ValidationComplete");
            Ok(StateHandlerOutcome::transition(
                SwitchControllerState::BomValidating {
                    bom_validating_state: BomValidatingState::BomValidationComplete,
                },
            ))
        }
    }
}
