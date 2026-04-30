/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge as rpc;
use serde::{Deserialize, Serialize};

use crate::rpc_error::PxeRequestError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MachineArchitecture {
    Arm = 0,
    X86 = 1,
}

impl TryFrom<&str> for MachineArchitecture {
    type Error = PxeRequestError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "arm64" => Ok(MachineArchitecture::Arm),
            "x86_64" => Ok(MachineArchitecture::X86),
            x if x == (MachineArchitecture::Arm as u64).to_string().as_str() => {
                Ok(MachineArchitecture::Arm)
            }
            x if x == (MachineArchitecture::X86 as u64).to_string().as_str() => {
                Ok(MachineArchitecture::X86)
            }
            _ => Err(PxeRequestError::MalformedBuildArch(format!(
                "Not a valid architecture identifier: {value}"
            ))),
        }
    }
}

impl From<MachineArchitecture> for rpc::MachineArchitecture {
    fn from(arch: MachineArchitecture) -> rpc::MachineArchitecture {
        match arch {
            MachineArchitecture::X86 => rpc::MachineArchitecture::X86,
            MachineArchitecture::Arm => rpc::MachineArchitecture::Arm,
        }
    }
}
