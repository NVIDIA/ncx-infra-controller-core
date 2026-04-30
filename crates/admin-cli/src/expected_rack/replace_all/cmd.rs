/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use ::rpc::forge as rpc_forge;
use serde::{Deserialize, Serialize};

use super::Args;
use crate::expected_rack::common::ExpectedRackJson;
use crate::rpc::ApiClient;

#[derive(Debug, Serialize, Deserialize)]
struct ExpectedRackList {
    expected_racks: Vec<ExpectedRackJson>,
    expected_racks_count: Option<usize>,
}

/// replace_all clears all expected racks and replaces them with the contents of a JSON file.
pub async fn replace_all(args: Args, api_client: &ApiClient) -> CarbideCliResult<()> {
    let json_file_path = Path::new(&args.filename);
    let reader = BufReader::new(File::open(json_file_path)?);

    let expected_rack_list: ExpectedRackList = serde_json::from_reader(reader)?;

    if expected_rack_list
        .expected_racks_count
        .is_some_and(|count| count != expected_rack_list.expected_racks.len())
    {
        return Err(CarbideCliError::GenericError(format!(
            "Json File specified an invalid count: {:#?}; actual count: {}",
            expected_rack_list.expected_racks_count.unwrap_or_default(),
            expected_rack_list.expected_racks.len()
        )));
    }

    let request = rpc_forge::ExpectedRackList {
        expected_racks: expected_rack_list
            .expected_racks
            .into_iter()
            .map(|rack| rpc_forge::ExpectedRack {
                rack_id: Some(rack.rack_id),
                rack_profile_id: Some(rack.rack_profile_id),
                metadata: rack.metadata,
            })
            .collect(),
    };

    api_client
        .0
        .replace_all_expected_racks(request)
        .await
        .map_err(CarbideCliError::ApiInvocationError)?;
    Ok(())
}
