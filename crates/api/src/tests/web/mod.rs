/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use axum::Router;
use common::api_fixtures::TestEnv;
use hyper::http::Request;
use hyper::http::request::Builder;

use crate::tests::common;
use crate::web::routes;
mod machine_health;
mod managed_host;
mod vpc;

fn make_test_app(env: &TestEnv) -> Router {
    let r = routes(env.api.clone()).unwrap();
    Router::new().nest_service("/admin", r)
}

/// Builder for admin UI requests (in-process auth defaults to none in tests).
fn web_request_builder() -> Builder {
    Request::builder().header("Host", "with.the.most")
}
