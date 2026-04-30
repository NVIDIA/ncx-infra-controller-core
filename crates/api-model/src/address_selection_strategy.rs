/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[derive(Clone, Copy)]
pub enum AddressSelectionStrategy {
    /// Allocate the next available single IP address.
    /// Uses /32 for IPv4 prefixes, /128 for IPv6 prefixes.
    NextAvailableIp,

    /// Alias for `NextAvailableIp`. Kept for backwards compatibility.
    Automatic,

    /// Allocate the next available prefix of the given length.
    /// For example, `NextAvailablePrefix(30)` allocates a /30 block
    /// (used by FNN to allocate a 4-address subnet per DPU).
    NextAvailablePrefix(u8),

    /// Assign a specific IP address to the interface.
    ///
    /// This IP address can either be a "reservation" within an
    /// existing carbide-dhcp managed network (and allows you
    /// to pin your device to an IP within a managed network),
    /// or it can be outside of the Carbide-managed networks
    /// entirely, allowing you to effectively BYO DHCP for
    /// underlay interfaces.
    StaticAddress(std::net::IpAddr),
}
