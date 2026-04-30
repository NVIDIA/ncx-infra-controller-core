/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge as rpc;
use tonic::{Request, Response, Status};

use crate::api::{Api, log_request_data};

pub(crate) async fn trim_table(
    api: &Api,
    request: Request<rpc::TrimTableRequest>,
) -> Result<Response<rpc::TrimTableResponse>, Status> {
    log_request_data(&request);

    let mut txn = api.txn_begin().await?;

    let target: model::trim_table::TrimTableTarget = request.get_ref().target().into();
    let total_deleted =
        db::trim_table::trim_table(&mut txn, target, request.get_ref().keep_entries).await?;

    txn.commit().await?;

    Ok(Response::new(rpc::TrimTableResponse {
        total_deleted: total_deleted.to_string(),
    }))
}
