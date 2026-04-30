/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handler for SwitchControllerState::Ready.

use carbide_uuid::switch::SwitchId;
use model::switch::{ReProvisioningState, Switch, SwitchControllerState};

use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};
use crate::state_controller::switch::context::SwitchStateHandlerContextObjects;

/// Handles the Ready state for a switch.
/// TODO: Implement Switch monitoring (health checks, status updates, etc.).
pub async fn handle_ready(
    _switch_id: &SwitchId,
    state: &mut Switch,
    _ctx: &mut StateHandlerContext<'_, SwitchStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<SwitchControllerState>, StateHandlerError> {
    if state.is_marked_as_deleted() {
        return Ok(StateHandlerOutcome::transition(
            SwitchControllerState::Deleting,
        ));
    }

    if let Some(req) = &state.switch_reprovisioning_requested {
        if req.initiator.starts_with("rack-") {
            tracing::info!(
                "Rack-level firmware upgrade requested — transitioning to WaitingForRackFirmwareUpgrade"
            );
            return Ok(StateHandlerOutcome::transition(
                SwitchControllerState::ReProvisioning {
                    reprovisioning_state: ReProvisioningState::WaitingForRackFirmwareUpgrade,
                },
            ));
        }

        tracing::warn!(
            "unknown initiator for switch reprovisioning request: {}",
            req.initiator
        );
        return Ok(StateHandlerOutcome::transition(
            SwitchControllerState::Error {
                cause: format!(
                    "unknown initiator for switch reprovisioning request: {}",
                    req.initiator
                ),
            },
        ));
    }

    tracing::info!("Switch is ready");
    Ok(StateHandlerOutcome::do_nothing())
}
