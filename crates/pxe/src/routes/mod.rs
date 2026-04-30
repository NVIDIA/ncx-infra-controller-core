/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::forge as rpc;
use ::rpc::forge_tls_client::{self, ApiConfig, ForgeClientConfig};
use carbide_uuid::machine::MachineInterfaceId;

pub(crate) mod cloud_init;
pub(crate) mod ipxe;
pub(crate) mod metrics;
pub(crate) mod tls;

pub struct RpcContext;

impl RpcContext {
    async fn get_pxe_instructions(
        arch: rpc::MachineArchitecture,
        interface_id: MachineInterfaceId,
        product: Option<String>,
        url: &str,
        client_config: &ForgeClientConfig,
    ) -> Result<rpc::PxeInstructions, String> {
        let api_config = ApiConfig::new(url, client_config);
        let mut client = forge_tls_client::ForgeTlsClient::retry_build(&api_config)
            .await
            .map_err(|err| err.to_string())?;
        let request = tonic::Request::new(rpc::PxeInstructionRequest {
            arch: arch as i32,
            interface_id: Some(interface_id),
            product,
        });
        client
            .get_pxe_instructions(request)
            .await
            .map(|response| response.into_inner())
            .map_err(|error| {
                format!(
                    "Error in updating build needed flag for instance for machine {interface_id:?}; Error: {error}."
                )
            })
    }
}
