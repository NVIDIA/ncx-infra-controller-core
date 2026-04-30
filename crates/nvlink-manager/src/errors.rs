/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use db::DatabaseError;
use rpc::errors::RpcDataConversionError;

#[derive(thiserror::Error, Debug)]
pub enum NvLinkManagerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Can not convert between RPC data model and internal data model - {0}")]
    RpcDataConversionError(#[from] RpcDataConversionError),
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl NvLinkManagerError {
    /// Creates a `Internal` error with the given error message
    pub fn internal(message: String) -> Self {
        Self::Internal { message }
    }
}

pub type NvLinkManagerResult<T> = Result<T, NvLinkManagerError>;
