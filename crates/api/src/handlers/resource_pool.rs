/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::collections::HashMap;

use ::rpc::forge as rpc;
use tonic::{Request, Response, Status};

use crate::CarbideError;
use crate::api::Api;

pub(crate) async fn grow(
    api: &Api,
    request: Request<rpc::GrowResourcePoolRequest>,
) -> Result<Response<rpc::GrowResourcePoolResponse>, Status> {
    crate::api::log_request_data(&request);

    let toml_text = request.into_inner().text;

    let mut txn = api.txn_begin().await?;

    let mut pools = HashMap::new();
    let table: toml::Table = toml_text
        .parse()
        .map_err(|e: toml::de::Error| CarbideError::InvalidArgument(e.to_string()))?;
    for (name, def) in table {
        let d: model::resource_pool::ResourcePoolDef = def
            .try_into()
            .map_err(|e: toml::de::Error| CarbideError::InvalidArgument(e.to_string()))?;
        pools.insert(name, d);
    }
    use db::resource_pool::DefineResourcePoolError as DE;
    match db::resource_pool::define_all_from(&mut txn, &pools).await {
        Ok(()) => {
            txn.commit().await?;
            Ok(Response::new(rpc::GrowResourcePoolResponse {}))
        }
        Err(DE::InvalidArgument(msg)) => Err(CarbideError::InvalidArgument(msg).into()),
        Err(DE::InvalidToml(err)) => Err(CarbideError::InvalidArgument(err.to_string()).into()),
        Err(DE::ResourcePoolError(msg)) => Err(CarbideError::Internal {
            message: msg.to_string(),
        }
        .into()),
        Err(DE::DatabaseError(err)) => Err(CarbideError::Internal {
            message: err.to_string(),
        }
        .into()),
        Err(err @ DE::TooBig(_, _)) => Err(tonic::Status::out_of_range(err.to_string())),
    }
}

pub(crate) async fn list(
    api: &Api,
    request: Request<rpc::ListResourcePoolsRequest>,
) -> Result<tonic::Response<rpc::ResourcePools>, tonic::Status> {
    crate::api::log_request_data(&request);

    let mut txn = api.txn_begin().await?;

    let snapshot = db::resource_pool::all(&mut txn)
        .await
        .map_err(CarbideError::from)?;

    txn.commit().await?;

    Ok(Response::new(rpc::ResourcePools {
        pools: snapshot.into_iter().map(|s| s.into()).collect(),
    }))
}
