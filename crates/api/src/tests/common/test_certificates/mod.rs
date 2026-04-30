/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::collections::HashMap;

use async_trait::async_trait;
use forge_secrets::SecretsError;
use forge_secrets::certificates::{Certificate, CertificateProvider};
use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct TestCertificateProvider {
    pub certificates: Mutex<HashMap<String, Certificate>>,
}

impl TestCertificateProvider {
    pub fn new() -> Self {
        Self {
            certificates: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl CertificateProvider for TestCertificateProvider {
    async fn get_certificate(
        &self,
        unique_identifier: &str,
        _alt_names: Option<String>,
        _ttl: Option<String>,
    ) -> Result<Certificate, SecretsError> {
        let mut certificates = self.certificates.lock().await;
        let certificate = certificates
            .entry(unique_identifier.to_string())
            .or_insert(Certificate::default());

        Ok(certificate.clone())
    }
}
