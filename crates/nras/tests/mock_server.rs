/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub enum Method {
    Get,
    Post,
}

impl Method {
    fn to_string(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
        }
    }
}

pub fn add_mock(
    server: &mut mockito::ServerGuard,
    path: &str,
    response_body: &str,
    method: &Method,
    status_code: usize,
) -> String {
    // Create a mock
    server
        .mock(method.to_string(), path)
        .with_status(status_code)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create();

    format!("{}{}", server.url(), path)
}

pub async fn create_mock_http_server() -> mockito::ServerGuard {
    // Request a new server from the pool
    mockito::Server::new_async().await
}
