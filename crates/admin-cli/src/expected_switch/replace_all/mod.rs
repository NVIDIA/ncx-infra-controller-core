/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod args;
pub mod cmd;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult};
pub use args::Args;
use serde::{Deserialize, Serialize};

use crate::cfg::run::Run;
use crate::cfg::runtime::RuntimeContext;
use crate::expected_switch::common::ExpectedSwitchJson;

impl Run for Args {
    async fn run(self, ctx: &mut RuntimeContext) -> CarbideCliResult<()> {
        let json_file_path = Path::new(&self.filename);
        let reader = BufReader::new(File::open(json_file_path)?);

        #[derive(Debug, Serialize, Deserialize)]
        struct ExpectedSwitchList {
            expected_switches: Vec<ExpectedSwitchJson>,
            expected_switches_count: Option<usize>,
        }

        let expected_switch_list: ExpectedSwitchList = serde_json::from_reader(reader)?;

        if expected_switch_list
            .expected_switches_count
            .is_some_and(|count| count != expected_switch_list.expected_switches.len())
        {
            return Err(CarbideCliError::GenericError(format!(
                "Json File specified an invalid count: {:#?}; actual count: {}",
                expected_switch_list
                    .expected_switches_count
                    .unwrap_or_default(),
                expected_switch_list.expected_switches.len()
            )));
        }

        ctx.api_client
            .replace_all_expected_switches(expected_switch_list.expected_switches)
            .await?;
        Ok(())
    }
}
