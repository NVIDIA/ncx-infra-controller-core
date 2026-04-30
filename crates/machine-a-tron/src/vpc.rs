/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fmt::Debug;
use std::sync::Arc;

use ::rpc::Timestamp;
use ::rpc::forge::VpcVirtualizationType;
use carbide_uuid::vpc::VpcId;

use crate::config::MachineATronContext;
use crate::tui::{UiUpdate, VpcDetails};

#[derive(Debug, Clone)]
pub struct Vpc {
    pub vpc_id: VpcId,
    pub app_context: Arc<MachineATronContext>,

    pub vpc_name: String,
    pub network_virtualization_type: Option<VpcVirtualizationType>,

    pub logs: Vec<String>,

    _created: Option<Timestamp>,
}

impl Vpc {
    pub async fn new(
        app_context: Arc<MachineATronContext>,
        ui_event_tx: Option<tokio::sync::mpsc::Sender<UiUpdate>>,
        network_virtualization_type: Option<VpcVirtualizationType>,
    ) -> Self {
        // TODO: Add error handling when vpc creation fails.
        let vpc = app_context
            .api_client()
            .create_vpc(network_virtualization_type)
            .await
            .unwrap();

        let new_vpc = Vpc {
            vpc_id: vpc.id.expect("VPC must have an ID."),
            app_context,
            vpc_name: vpc.name,
            network_virtualization_type,
            logs: Vec::default(),
            _created: vpc.created,
        };

        let details = VpcDetails::from(&new_vpc);
        if let Some(ui_event_tx) = ui_event_tx.as_ref() {
            _ = ui_event_tx
                .send(UiUpdate::Vpc(details))
                .await
                .inspect_err(|e| tracing::warn!("Error sending TUI event: {}", e));
        }

        new_vpc
    }
}
