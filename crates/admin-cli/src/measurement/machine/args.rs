/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/*!
 *  Measured Boot CLI arguments for the `measurement mock-machine` subcommand.
 *
 * This provides the CLI subcommands and arguments for:
 *  - `mock-machine create`: Creates a new "mock" machine.
 *  - `mock-machine delete`: Deletes an existing mock machine.
 *  - `mock-machine attest`: Sends a measurement report for a mock machine.
 *  - `mock-machine show [id]`: Shows detailed info about mock machine(s).
 *  - `mock-machine list``: Lists all mock machines.
 */

use ::rpc::admin_cli::CarbideCliError;
use ::rpc::protos::measured_boot::{
    AttestCandidateMachineRequest, ShowCandidateMachineRequest, show_candidate_machine_request,
};
use carbide_uuid::machine::MachineId;
use clap::Parser;
use measured_boot::pcr::PcrRegisterValue;

use crate::cfg::measurement::parse_pcr_register_values;

/// CmdMachine provides a container for the `mock-machine`
/// subcommand, which itself contains other subcommands
/// for working with mock machines.
#[derive(Parser, Debug)]
pub enum CmdMachine {
    #[clap(about = "Send measurements for a machine.", visible_alias = "a")]
    Attest(Attest),

    #[clap(about = "Get all info about a machine.", visible_alias = "s")]
    Show(Show),

    #[clap(about = "List all machines + their info.", visible_alias = "l")]
    List(List),
}

/// Attest sends a measurement report for the given machine ID,
/// where the measurement report then goes through attestation in an
/// attempt to match a bundle.
#[derive(Parser, Debug)]
pub struct Attest {
    #[clap(help = "The machine ID of the machine to associate this report with.")]
    pub machine_id: MachineId,

    #[clap(
        required = true,
        use_value_delimiter = true,
        value_delimiter = ',',
        help = "Comma-separated list of {pcr_register:value,...} to associate with this report."
    )]
    #[arg(value_parser = parse_pcr_register_values)]
    pub values: Vec<PcrRegisterValue>,
}

/// List lists all candidate machines.
#[derive(Parser, Debug)]
pub struct List {}

/// Show will get a candidate machine for the given ID, or all machines
/// if no machine ID is provided.
#[derive(Parser, Debug)]
pub struct Show {
    #[clap(help = "The machine ID to show.")]
    pub machine_id: Option<MachineId>,
}

impl From<Attest> for AttestCandidateMachineRequest {
    fn from(attest: Attest) -> Self {
        Self {
            machine_id: attest.machine_id.to_string(),
            pcr_values: attest.values.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<Show> for ShowCandidateMachineRequest {
    type Error = CarbideCliError;
    fn try_from(show: Show) -> Result<Self, Self::Error> {
        let machine_id = show
            .machine_id
            .ok_or(CarbideCliError::GenericError(String::from(
                "machine_id must be set to get a machine",
            )))?;
        Ok(Self {
            selector: Some(show_candidate_machine_request::Selector::MachineId(
                machine_id.to_string(),
            )),
        })
    }
}
