/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use serde::{Deserialize, Serialize};

use crate::address_selection_strategy::AddressSelectionStrategy;

/// Distinguishes how an IP address was allocated to a machine interface,
/// and are generally derived from the AddressSelectionStrategy used.
///
/// - `Dhcp`: These addresses allocated and managed by carbide-dhcp,
///   or a DHCP service that integrates directly with carbide-api.
/// - `Static`: These addresses are assigned and managed explicitly by
///   an operator or operator-provided configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AllocationType {
    Dhcp,
    Static,
}

impl From<AddressSelectionStrategy> for AllocationType {
    fn from(strategy: AddressSelectionStrategy) -> Self {
        match strategy {
            AddressSelectionStrategy::NextAvailableIp => AllocationType::Dhcp,
            AddressSelectionStrategy::Automatic => AllocationType::Dhcp,
            AddressSelectionStrategy::NextAvailablePrefix(_) => AllocationType::Dhcp,
            AddressSelectionStrategy::StaticAddress(_) => AllocationType::Static,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn next_available_ip_is_dhcp() {
        assert_eq!(
            AllocationType::from(AddressSelectionStrategy::NextAvailableIp),
            AllocationType::Dhcp,
        );
    }

    #[test]
    fn automatic_is_dhcp() {
        assert_eq!(
            AllocationType::from(AddressSelectionStrategy::Automatic),
            AllocationType::Dhcp,
        );
    }

    #[test]
    fn next_available_prefix_is_dhcp() {
        assert_eq!(
            AllocationType::from(AddressSelectionStrategy::NextAvailablePrefix(30)),
            AllocationType::Dhcp,
        );
    }

    #[test]
    fn static_address_is_static() {
        assert_eq!(
            AllocationType::from(AddressSelectionStrategy::StaticAddress(
                Ipv4Addr::new(10, 0, 0, 1).into()
            )),
            AllocationType::Static,
        );
    }

    #[test]
    fn serde_roundtrip() {
        let dhcp: AllocationType = serde_json::from_str(r#""dhcp""#).unwrap();
        assert_eq!(dhcp, AllocationType::Dhcp);

        let static_: AllocationType = serde_json::from_str(r#""static""#).unwrap();
        assert_eq!(static_, AllocationType::Static);

        assert_eq!(
            serde_json::to_string(&AllocationType::Dhcp).unwrap(),
            r#""dhcp""#
        );
        assert_eq!(
            serde_json::to_string(&AllocationType::Static).unwrap(),
            r#""static""#
        );
    }
}
