/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;
use rpc::admin_cli::{CarbideCliError, CarbideCliResult};

#[derive(Parser, Debug)]
pub struct DpfQuery {
    #[clap(help = "Host machine id")]
    pub host: Option<MachineId>,
}

impl TryFrom<&DpfQuery> for MachineId {
    type Error = CarbideCliError;

    fn try_from(query: &DpfQuery) -> CarbideCliResult<Self> {
        let Some(host) = query.host else {
            return Err(CarbideCliError::GenericError(
                "Host id is required!!".to_string(),
            ));
        };

        if host.machine_type() == carbide_uuid::machine::MachineType::Dpu {
            return Err(CarbideCliError::GenericError(
                "Only host id is expected!!".to_string(),
            ));
        }

        Ok(host)
    }
}
