/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handler for SwitchControllerState::Created.

use carbide_uuid::switch::SwitchId;
use model::switch::{InitializingState, Switch, SwitchControllerState};

use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};
use crate::state_controller::switch::context::SwitchStateHandlerContextObjects;

pub async fn handle_created(
    switch_id: &SwitchId,
    _state: &mut Switch,
    _ctx: &mut StateHandlerContext<'_, SwitchStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<SwitchControllerState>, StateHandlerError> {
    tracing::info!(
        "Switch {:?} created, transitioning to Initializing",
        switch_id
    );
    Ok(StateHandlerOutcome::transition(
        SwitchControllerState::Initializing {
            initializing_state: InitializingState::WaitForOsMachineInterface,
        },
    ))
}
