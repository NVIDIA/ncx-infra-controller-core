/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use forge_secrets::credentials::{BmcCredentialType, CredentialKey};
use mac_address::MacAddress;

pub enum RedfishAuth {
    Anonymous,
    Key(CredentialKey),
    Direct(String, String), // username, password
}

impl RedfishAuth {
    pub fn for_bmc_mac(bmc_mac_address: MacAddress) -> Self {
        RedfishAuth::Key(CredentialKey::BmcCredentials {
            // TODO(ajf): Change this to Forge Admin user once site explorer
            // ensures it exist, credentials are done by mac address
            credential_type: BmcCredentialType::BmcRoot { bmc_mac_address },
        })
    }
}
