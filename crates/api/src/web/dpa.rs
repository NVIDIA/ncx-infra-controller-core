/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;

use askama::Template;
use axum::Json;
use axum::extract::{Path as AxumPath, State as AxumState};
use axum::response::{Html, IntoResponse, Response};
use hyper::http::StatusCode;
use rpc::forge as forgerpc;
use rpc::forge::forge_server::Forge;
use uuid::Uuid;

use super::filters;
use super::state_history::StateHistoryTable;
use crate::api::Api;

#[derive(Template)]
#[template(path = "dpa_show.html")]
struct DpaShow {
    dpas: Vec<forgerpc::DpaInterface>,
}

/// List DPAs
pub async fn show_dpas_html(AxumState(state): AxumState<Arc<Api>>) -> Response {
    let dpas = match fetch_dpas(state.clone()).await {
        Ok(n) => n,
        Err(err) => {
            tracing::error!(%err, "fetch_dpas");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error loading dpas").into_response();
        }
    };

    let tmpl = DpaShow { dpas };
    (StatusCode::OK, Html(tmpl.render().unwrap())).into_response()
}

pub async fn show_dpas_json(AxumState(state): AxumState<Arc<Api>>) -> Response {
    let dpas = match fetch_dpas(state).await {
        Ok(n) => n,
        Err(err) => {
            tracing::error!(%err, "fetch_dpas");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error loading DPAs").into_response();
        }
    };
    (StatusCode::OK, Json(dpas)).into_response()
}

async fn fetch_dpas(api: Arc<Api>) -> Result<Vec<forgerpc::DpaInterface>, tonic::Status> {
    let request = tonic::Request::new(());

    let dpa_ids = api
        .get_all_dpa_interface_ids(request)
        .await?
        .into_inner()
        .ids;

    let mut dpas = Vec::new();
    let mut offset = 0;
    while offset != dpa_ids.len() {
        const PAGE_SIZE: usize = 100;
        let page_size = PAGE_SIZE.min(dpa_ids.len() - offset);
        let next_ids = &dpa_ids[offset..offset + page_size];
        let request = tonic::Request::new(forgerpc::DpaInterfacesByIdsRequest {
            ids: next_ids.to_vec(),
            include_history: false,
        });
        let next_dpas = api
            .find_dpa_interfaces_by_ids(request)
            .await
            .map(|response| response.into_inner())?;

        dpas.extend(next_dpas.interfaces.into_iter());
        offset += page_size;
    }

    dpas.sort_unstable_by(|dpa1, dpa2| dpa1.id.cmp(&dpa2.id));

    Ok(dpas)
}

#[derive(Template)]
#[template(path = "dpa_detail.html")]
struct DpaDetail {
    dpa: forgerpc::DpaInterface,
    history: StateHistoryTable,
}

impl From<forgerpc::DpaInterface> for DpaDetail {
    fn from(dpa: forgerpc::DpaInterface) -> Self {
        let history = StateHistoryTable {
            records: dpa.history.iter().rev().cloned().map(Into::into).collect(),
        };

        Self { dpa, history }
    }
}

/// View DPA details
pub async fn detail(
    AxumState(state): AxumState<Arc<Api>>,
    AxumPath(dpa_id): AxumPath<String>,
) -> Response {
    let (show_json, dpa_id) = match dpa_id.strip_suffix(".json") {
        Some(dpa_id) => (true, dpa_id.to_string()),
        None => (false, dpa_id),
    };

    let dpaid = match Uuid::parse_str(&dpa_id) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DPA id {dpa_id} could not be parsed into UUID Err {e}"),
            )
                .into_response();
        }
    };

    let request = tonic::Request::new(forgerpc::DpaInterfacesByIdsRequest {
        ids: vec![dpaid.into()],
        include_history: true,
    });
    let dpa = match state
        .find_dpa_interfaces_by_ids(request)
        .await
        .map(|response| response.into_inner())
    {
        Ok(l) if l.interfaces.is_empty() => {
            return super::not_found_response(dpa_id);
        }
        Ok(l) if l.interfaces.len() != 1 => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DPA list for {dpa_id} returned {} DPAs", l.interfaces.len()),
            )
                .into_response();
        }
        Ok(mut l) => l.interfaces.remove(0),
        Err(err) if err.code() == tonic::Code::NotFound => {
            return super::not_found_response(dpa_id);
        }
        Err(err) => {
            tracing::error!(%err, "find_dpas_by_ids");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error loading DPAs").into_response();
        }
    };

    if show_json {
        return (StatusCode::OK, Json(dpa)).into_response();
    }

    let tmpl: DpaDetail = dpa.into();
    (StatusCode::OK, Html(tmpl.render().unwrap())).into_response()
}
