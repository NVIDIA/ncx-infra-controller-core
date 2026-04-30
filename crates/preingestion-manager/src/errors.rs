/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use db::DatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum PreingestionManagerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Error in libredfish: {0}")]
    RedfishError(#[from] libredfish::RedfishError),
    #[error("Argument is invalid: {0}")]
    InvalidArgument(String),
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl PreingestionManagerError {
    /// Creates a `Internal` error with the given error message
    pub fn internal(message: String) -> Self {
        Self::Internal { message }
    }
}

pub type PreingestionManagerResult<T> = Result<T, PreingestionManagerError>;
