/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::time::Duration;

use axum::http::StatusCode;
use bmc_explorer::ErrorClass;
use bmc_mock::test_support::TestBmc;
use bmc_mock::test_support::axum_http_client::Error as TestBmcError;

pub fn error_classifier(err: &<TestBmc as nv_redfish::Bmc>::Error) -> Option<ErrorClass> {
    match err {
        TestBmcError::InvalidResponse { status, .. } => match *status {
            StatusCode::NOT_FOUND => Some(ErrorClass::NotFound),
            StatusCode::INTERNAL_SERVER_ERROR => Some(ErrorClass::InternalServerError),
            _ => None,
        },
        _ => None,
    }
}

pub fn explorer_config() -> bmc_explorer::Config<'static, TestBmc> {
    bmc_explorer::Config {
        boot_interface_mac: None,
        error_classifier: &error_classifier,
        retry_timeout: Duration::from_millis(0),
    }
}
