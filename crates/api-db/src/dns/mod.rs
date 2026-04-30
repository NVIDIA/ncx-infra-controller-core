/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod domain;
pub mod domain_metadata;
pub mod resource_record;

pub fn normalize_domain(name: &str) -> String {
    let normalize_domain = name.trim_end_matches('.').to_lowercase();
    tracing::debug!("Normalized domain name: {} to: {}", name, normalize_domain);
    normalize_domain
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_normalize_domain_name() {
        let domain_name = "example.com.";
        let expected = "example.com";
        let normalized = super::normalize_domain(domain_name);
        assert_eq!(normalized, expected);
    }
}
