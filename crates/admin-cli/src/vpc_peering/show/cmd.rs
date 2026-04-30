/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::output::OutputFormat;
use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use rpc::forge::{VpcPeeringIdList, VpcPeeringSearchFilter, VpcPeeringsByIdsRequest};

use super::args::Args;
use crate::rpc::ApiClient;
use crate::vpc_peering::convert_vpc_peerings_to_table;

pub async fn show(
    args: &Args,
    output_format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let is_json = output_format == OutputFormat::Json;

    let vpc_peering_ids = match (&args.id, &args.vpc_id) {
        (Some(id), None) => VpcPeeringIdList {
            vpc_peering_ids: vec![*id],
        },
        (None, _) => {
            api_client
                .0
                .find_vpc_peering_ids(VpcPeeringSearchFilter {
                    vpc_id: args.vpc_id,
                })
                .await?
        }
        _ => unreachable!(
            "`--id` and `--vpc-id` are mutually exclusive and enforced by clap via `conflicts_with`"
        ),
    };

    let vpc_peering_list = api_client
        .0
        .find_vpc_peerings_by_ids(VpcPeeringsByIdsRequest {
            vpc_peering_ids: vpc_peering_ids.vpc_peering_ids,
        })
        .await?;

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&vpc_peering_list).map_err(CarbideCliError::JsonError)?
        );
    } else {
        convert_vpc_peerings_to_table(&vpc_peering_list.vpc_peerings)?.printstd();
    }

    Ok(())
}
