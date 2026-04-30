/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::admin_cli::{CarbideCliError, CarbideCliResult};
use rpc::forge::AssociateMachinesWithInstanceTypeRequest;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "InstanceTypeId")]
    pub instance_type_id: String,
    #[clap(help = "Machine Ids, separated by comma", value_delimiter = ',')]
    pub machine_ids: Vec<String>,
}

impl TryFrom<Args> for AssociateMachinesWithInstanceTypeRequest {
    type Error = CarbideCliError;

    fn try_from(args: Args) -> CarbideCliResult<Self> {
        if args.machine_ids.is_empty() {
            return Err(CarbideCliError::GenericError(
                "Machine ids can not be empty.".to_string(),
            ));
        }

        Ok(AssociateMachinesWithInstanceTypeRequest {
            instance_type_id: args.instance_type_id,
            machine_ids: args.machine_ids,
        })
    }
}
