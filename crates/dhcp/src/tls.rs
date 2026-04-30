/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use forge_tls::client_config::ClientCert;
use rpc::forge_tls_client::ForgeClientConfig;

use crate::CONFIG;

pub fn build_forge_client_config() -> ForgeClientConfig {
    let forge_root_ca_path = &CONFIG
        .read()
        .unwrap() // safety: the only way this will panic is if the lock is poisoned,
        // which happens when another holder panics. we're already done at that point.
        .forge_root_ca_path;
    let forge_client_key_path = &CONFIG
        .read()
        .unwrap() // safety: the only way this will panic is if the lock is poisoned,
        // which happens when another holder panics. we're already done at that point.
        .forge_client_key_path;
    let forge_client_cert_path = &CONFIG
        .read()
        .unwrap() // safety: the only way this will panic is if the lock is poisoned,
        // which happens when another holder panics. we're already done at that point.
        .forge_client_cert_path;

    let client_cert = ClientCert {
        cert_path: forge_client_cert_path.clone(),
        key_path: forge_client_key_path.clone(),
    };

    ForgeClientConfig::new(forge_root_ca_path.clone(), Some(client_cert))
}
