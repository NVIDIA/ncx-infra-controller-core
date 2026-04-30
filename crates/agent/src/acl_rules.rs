/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Path to the legacy ETV ACL rules file (used by cleanup_old_acls)
pub const PATH: &str = "etc/cumulus/acl/policy.d/60-forge.rules";

/// Command to reload ACL rules
pub const RELOAD_CMD: &str = "cl-acltool -i";

/// ACL to suppress ARP packets before encapsulation
pub const ARP_SUPPRESSION_RULE: &str = r"
[ebtables]
# Suppress ARP packets before they get encapsulated.
-A OUTPUT -o vxlan48 -p ARP -j DROP
";
