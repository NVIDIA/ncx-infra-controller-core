/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::net::SocketAddr;

use libredfish::RoleId;
use libredfish::model::oem::nvidia_dpu::NicMode;
use mac_address::MacAddress;
use model::expected_entity::ExpectedEntity;
use model::machine::MachineInterfaceSnapshot;
use model::site_explorer::{EndpointExplorationError, EndpointExplorationReport, LockdownStatus};

use super::metrics::SiteExplorationMetrics;

/// This trait defines how the `SiteExplorer` will query information about endpoints
#[async_trait::async_trait]
pub trait EndpointExplorer: Send + Sync + 'static {
    /// Query an endpoint for information
    ///
    /// The query carries the information `MachineInterface` information that is derived
    /// from DHCP requests as well as the information that might have been fetched in
    /// a previous exploration.
    async fn explore_endpoint(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        expected: Option<&ExpectedEntity>,
        last_report: Option<&EndpointExplorationReport>,
        boot_interface_mac: Option<MacAddress>,
    ) -> Result<EndpointExplorationReport, EndpointExplorationError>;

    async fn check_preconditions(
        &self,
        metrics: &mut SiteExplorationMetrics,
    ) -> Result<(), EndpointExplorationError>;

    // redfish_reset_bmc issues a BMC reset through redfish.
    async fn redfish_reset_bmc(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<(), EndpointExplorationError>;

    // ipmitool_reset_bmc issues a cold BMC reset through ipmitool.
    async fn ipmitool_reset_bmc(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<(), EndpointExplorationError>;

    async fn redfish_get_power_state(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<libredfish::PowerState, EndpointExplorationError>;

    async fn redfish_power_control(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        action: libredfish::SystemPowerControl,
    ) -> Result<(), EndpointExplorationError>;

    async fn have_credentials(&self, interface: &MachineInterfaceSnapshot) -> bool;

    async fn disable_secure_boot(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<(), EndpointExplorationError>;

    async fn lockdown(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        action: libredfish::EnabledDisabled,
    ) -> Result<(), EndpointExplorationError>;

    async fn lockdown_status(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<LockdownStatus, EndpointExplorationError>;

    async fn enable_infinite_boot(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<(), EndpointExplorationError>;

    async fn is_infinite_boot_enabled(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<Option<bool>, EndpointExplorationError>;

    async fn machine_setup(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        boot_interface_mac: Option<&str>,
    ) -> Result<(), EndpointExplorationError>;

    async fn set_boot_order_dpu_first(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        boot_interface_mac: &str,
    ) -> Result<(), EndpointExplorationError>;

    async fn set_nic_mode(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        mode: NicMode,
    ) -> Result<(), EndpointExplorationError>;

    async fn is_viking(
        &self,
        bmc_ip_address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<bool, EndpointExplorationError>;

    async fn clear_nvram(
        &self,
        bmc_ip_address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
    ) -> Result<(), EndpointExplorationError>;

    async fn create_bmc_user(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        username: &str,
        password: &str,
        role_id: RoleId,
    ) -> Result<(), EndpointExplorationError>;

    async fn delete_bmc_user(
        &self,
        address: SocketAddr,
        interface: &MachineInterfaceSnapshot,
        username: &str,
    ) -> Result<(), EndpointExplorationError>;
}
