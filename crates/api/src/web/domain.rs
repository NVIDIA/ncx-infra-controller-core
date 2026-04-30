/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;

use askama::Template;
use axum::Json;
use axum::extract::State as AxumState;
use axum::response::{Html, IntoResponse, Response};
use hyper::http::StatusCode;
use rpc::forge::forge_server::Forge;

use super::filters;
use crate::api::Api;

#[derive(Template)]
#[template(path = "domain_show.html")]
struct DomainShow {
    domains: Vec<::rpc::protos::dns::Domain>,
}

/// List domains
pub async fn show_html(AxumState(state): AxumState<Arc<Api>>) -> Response {
    let domains = match fetch_domains(state).await {
        Ok(m) => m,
        Err(err) => {
            tracing::error!(%err, "find_domains");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error loading domains").into_response();
        }
    };

    let tmpl = DomainShow {
        domains: domains.domains,
    };
    (StatusCode::OK, Html(tmpl.render().unwrap())).into_response()
}
pub async fn show_all_json(AxumState(state): AxumState<Arc<Api>>) -> Response {
    let domains = match fetch_domains(state).await {
        Ok(m) => m,
        Err(err) => {
            tracing::error!(%err, "find_domains");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error loading domains").into_response();
        }
    };
    (StatusCode::OK, Json(domains)).into_response()
}

async fn fetch_domains(api: Arc<Api>) -> Result<::rpc::protos::dns::DomainList, tonic::Status> {
    let request = tonic::Request::new(rpc::protos::dns::DomainSearchQuery {
        id: None,
        name: None,
    });
    api.find_domain(request)
        .await
        .map(|response| response.into_inner())
}
