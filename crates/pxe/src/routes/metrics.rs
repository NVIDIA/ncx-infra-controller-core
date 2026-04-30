/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use axum::Router;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;

use crate::common::AppState;

async fn metrics(state: State<AppState>) -> impl IntoResponse {
    // Make sure the metrics are fully updated prior to rendering them
    state.prometheus_handle.run_upkeep();

    state.prometheus_handle.render()
}

pub fn get_router(path_prefix: &str) -> Router<AppState> {
    Router::new().route(path_prefix, get(metrics))
}
