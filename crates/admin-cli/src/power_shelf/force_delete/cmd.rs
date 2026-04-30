/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use rpc::forge::AdminForceDeletePowerShelfRequest;

use super::args::Args;
use crate::rpc::ApiClient;

pub async fn force_delete(data: Args, api_client: &ApiClient) -> color_eyre::Result<()> {
    let response = api_client
        .0
        .admin_force_delete_power_shelf(AdminForceDeletePowerShelfRequest {
            power_shelf_id: Some(data.power_shelf_id),
            delete_interfaces: data.delete_interfaces,
        })
        .await?;

    println!(
        "Power shelf {} force deleted successfully.",
        response.power_shelf_id
    );
    if response.interfaces_deleted > 0 {
        println!("{} interface(s) deleted.", response.interfaces_deleted);
    }

    Ok(())
}
