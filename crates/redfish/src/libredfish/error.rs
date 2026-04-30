/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use forge_secrets::SecretsError;
use libredfish::RedfishError;

#[derive(thiserror::Error, Debug)]
pub enum RedfishClientCreationError {
    #[error("Missing credential {key}")]
    MissingCredentials { key: String },
    #[error("Missing credential: {cause}")]
    SecretEngineError { cause: SecretsError },
    #[error("Failed redfish request {0}")]
    RedfishError(RedfishError),
    #[error("Invalid Header {0}")]
    InvalidHeader(String),
    #[error("Missing Arguments: {0}")]
    MissingArgument(String),
    #[error("Missing BMC Information: {0}")]
    MissingBmcEndpoint(String),
    #[error("Database Error Loading Machine Interface")]
    MachineInterfaceLoadError(#[from] db::DatabaseError),
}

impl From<SecretsError> for RedfishClientCreationError {
    fn from(cause: SecretsError) -> Self {
        RedfishClientCreationError::SecretEngineError { cause }
    }
}
