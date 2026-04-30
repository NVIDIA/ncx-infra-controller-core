/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::expected_machine::ExpectedMachine;
use crate::expected_power_shelf::ExpectedPowerShelf;
use crate::expected_switch::ExpectedSwitch;

pub enum ExpectedEntity {
    Machine(ExpectedMachine),
    PowerShelf(ExpectedPowerShelf),
    Switch(ExpectedSwitch),
}

impl ExpectedEntity {
    pub fn bmc_credentials_data(&self) -> BmcCredentialsData<'_> {
        match self {
            Self::Machine(v) => BmcCredentialsData {
                username: &v.data.bmc_username,
                password: &v.data.bmc_password,
                retain_credentials: v.data.bmc_retain_credentials.unwrap_or(false),
            },
            Self::PowerShelf(v) => BmcCredentialsData {
                username: &v.bmc_username,
                password: &v.bmc_password,
                retain_credentials: v.bmc_retain_credentials.unwrap_or(false),
            },
            Self::Switch(v) => BmcCredentialsData {
                username: &v.bmc_username,
                password: &v.bmc_password,
                retain_credentials: v.bmc_retain_credentials.unwrap_or(false),
            },
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Machine(_) => "machine",
            Self::PowerShelf(_) => "power shelf",
            Self::Switch(_) => "switch",
        }
    }
}

#[derive(Clone, Copy)]
pub struct BmcCredentialsData<'a> {
    pub username: &'a str,
    pub password: &'a str,
    pub retain_credentials: bool,
}
