/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;

/// Replace all entries in the expected machines table with the entries from an inputted json file.
///
/// Example json file:
///    {
///        "expected_machines":
///        [
///            {
///                "bmc_mac_address": "1a:1b:1c:1d:1e:1f",
///                "bmc_username": "user",
///                "bmc_password": "pass",
///                "chassis_serial_number": "sample_serial-1"
///            },
///            {
///                "bmc_mac_address": "2a:2b:2c:2d:2e:2f",
///                "bmc_username": "user",
///                "bmc_password": "pass",
///                "chassis_serial_number": "sample_serial-2",
///                "fallback_dpu_serial_numbers": ["MT020100000003"],
///                "metadata": {
///                    "name": "MyMachine",
///                    "description": "My Machine",
///                    "labels": [{"key": "ABC", "value": "DEF"}]
///                }
///            }
///        ]
///    }
#[derive(Parser, Debug)]
#[clap(verbatim_doc_comment)]
pub struct Args {
    #[clap(short, long)]
    pub filename: String,
}
