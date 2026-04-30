/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use jsonwebtoken as jst;
use nras::{KeyStore, NrasError};

pub struct MockKeyStore {
    key: Option<jst::DecodingKey>,
}

impl MockKeyStore {
    pub fn new_with_key(x: &str, y: &str) -> MockKeyStore {
        let key = jst::DecodingKey::from_ec_components(x, y)
            .map_err(|e| NrasError::Jwk(format!("Error creating DecodingKey from EC PEM: {}", e)))
            .unwrap();

        MockKeyStore { key: Some(key) }
    }

    pub fn new_with_no_key() -> MockKeyStore {
        MockKeyStore { key: None }
    }
}

impl KeyStore for MockKeyStore {
    fn find_key(&self, _kid: &str) -> Option<jst::DecodingKey> {
        self.key.clone()
    }
}
