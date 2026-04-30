/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;

use carbide_uuid::machine::MachineInterfaceId;
use rpc::forge::forge_server::Forge;
use rpc::forge::{MachineArchitecture, PxeInstructions};

use crate::tests::common::api_fixtures::Api;

pub struct TestMachineInterface {
    id: MachineInterfaceId,
    api: Arc<Api>,
}

impl TestMachineInterface {
    pub fn new(id: MachineInterfaceId, api: Arc<Api>) -> Self {
        Self { id, api }
    }

    pub async fn get_pxe_instructions(&self, arch: MachineArchitecture) -> PxeInstructions {
        self.api
            .get_pxe_instructions(tonic::Request::new(rpc::forge::PxeInstructionRequest {
                arch: arch as i32,
                interface_id: Some(self.id),
                product: None,
            }))
            .await
            .unwrap()
            .into_inner()
    }
}
