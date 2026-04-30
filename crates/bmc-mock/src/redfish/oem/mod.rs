/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod dell;
pub mod nvidia;

use crate::redfish::Resource;

#[derive(Clone, Copy, Debug)]
pub enum BmcVendor {
    Dell,
    Nvidia(NvidiaNamestyle),
    Wiwynn,
    LiteOn,
    Ami,
}

#[derive(Clone, Copy, Debug)]
pub enum NvidiaNamestyle {
    Uppercase,
    Capitalized,
}

impl BmcVendor {
    pub fn service_root_value(&self) -> Option<&'static str> {
        match self {
            BmcVendor::Nvidia(NvidiaNamestyle::Capitalized) => Some("Nvidia"),
            BmcVendor::Nvidia(NvidiaNamestyle::Uppercase) => Some("NVIDIA"),
            BmcVendor::Dell => Some("Dell"),
            BmcVendor::Wiwynn => Some("WIWYNN"),
            BmcVendor::LiteOn => None,
            BmcVendor::Ami => Some("AMI"),
        }
    }
    // This function creates settings of the resource from the resource
    // id. Real identifier is different for different BMC vendors.
    pub fn make_settings_odata_id(&self, resource: &Resource<'_>) -> String {
        match self {
            BmcVendor::Nvidia(_) | BmcVendor::Dell | BmcVendor::Wiwynn | BmcVendor::LiteOn => {
                format!("{}/Settings", resource.odata_id)
            }
            BmcVendor::Ami => {
                format!("{}/SD", resource.odata_id)
            }
        }
    }
}

#[derive(Clone)]
pub enum State {
    NvidiaBluefield(nvidia::bluefield::BluefieldState),
    DellIdrac(dell::idrac::IdracState),
    Other,
}
