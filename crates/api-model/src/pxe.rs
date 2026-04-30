/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineInterfaceId;
use rpc::forge::MachineArchitecture;

pub struct PxeInstructionRequest {
    pub interface_id: MachineInterfaceId,
    pub arch: MachineArchitecture,
    pub product: Option<String>,
}

impl TryFrom<rpc::forge::PxeInstructionRequest> for PxeInstructionRequest {
    type Error = rpc::errors::RpcDataConversionError;

    fn try_from(value: rpc::forge::PxeInstructionRequest) -> Result<Self, Self::Error> {
        let interface_id =
            value
                .interface_id
                .ok_or(rpc::errors::RpcDataConversionError::MissingArgument(
                    "Interface ID",
                ))?;

        let arch = rpc::forge::MachineArchitecture::try_from(value.arch).map_err(|_| {
            rpc::errors::RpcDataConversionError::InvalidArgument(
                "Unknown arch received.".to_string(),
            )
        })?;

        let product = value.product;

        Ok(PxeInstructionRequest {
            interface_id,
            arch,
            product,
        })
    }
}
