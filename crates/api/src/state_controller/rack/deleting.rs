/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Handler for RackState::Deleting.

use model::rack::RackState;

use crate::state_controller::state_handler::{StateHandlerError, StateHandlerOutcome};

pub async fn handle_deleting() -> Result<StateHandlerOutcome<RackState>, StateHandlerError> {
    Ok(StateHandlerOutcome::wait("rack is being deleted".into()))
}
