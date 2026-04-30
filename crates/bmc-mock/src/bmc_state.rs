/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;

use crate::bug::InjectedBugs;
use crate::redfish;
use crate::redfish::chassis::ChassisState;
use crate::redfish::computer_system::SystemState;
use crate::redfish::manager::ManagerState;
use crate::redfish::update_service::UpdateServiceState;

#[derive(Clone)]
pub struct BmcState {
    pub bmc_vendor: redfish::oem::BmcVendor,
    pub bmc_product: Option<&'static str>,
    pub bmc_redfish_version: &'static str,
    pub oem_state: redfish::oem::State,
    pub manager: Arc<ManagerState>,
    pub system_state: Arc<SystemState>,
    pub chassis_state: Arc<ChassisState>,
    pub update_service_state: Arc<UpdateServiceState>,
    pub injected_bugs: Arc<InjectedBugs>,
    pub callbacks: Option<Arc<dyn crate::Callbacks>>,
}

#[derive(Clone, Copy, Debug)]
pub enum BmcEvent {
    PowerOn,
    BootCompleted,
}

impl BmcState {
    pub fn on_event(&self, event: &BmcEvent) {
        match event {
            BmcEvent::PowerOn => {
                self.complete_all_bios_jobs();
            }
            BmcEvent::BootCompleted => {
                self.system_state.on_boot_completed();
            }
        }
    }

    pub fn complete_all_bios_jobs(&self) {
        if let redfish::oem::State::DellIdrac(v) = &self.oem_state {
            v.complete_all_bios_jobs()
        }
    }
}
