/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::AdminForceDeleteSwitchRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn force_delete(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let response = api_client
        .0
        .admin_force_delete_switch(AdminForceDeleteSwitchRequest {
            switch_id: Some(data.switch_id),
            delete_interfaces: data.delete_interfaces,
        })
        .await?;

    println!("Switch {} force deleted successfully.", response.switch_id);
    if response.interfaces_deleted > 0 {
        println!("{} interface(s) deleted.", response.interfaces_deleted);
    }

    Ok(())
}
