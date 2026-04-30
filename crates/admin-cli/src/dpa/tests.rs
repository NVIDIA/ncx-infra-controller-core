/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// The intent of the tests.rs file is to test the integrity of the
// command, including things like basic structure parsing, enum
// translations, and any external input validators that are
// configured. Specific "categories" are:
//
// Command Structure - Baseline debug_assert() of the entire command.
// Argument Parsing  - Ensure required/optional arg combinations parse correctly.

use clap::{CommandFactory, Parser};

use super::*;

// verify_cmd_structure runs a baseline clap debug_assert()
// to do basic command configuration checking and validation,
// ensuring things like unique argument definitions, group
// configurations, argument references, etc. Things that would
// otherwise be missed until runtime.
#[test]
fn verify_cmd_structure() {
    Cmd::command().debug_assert();
}

/////////////////////////////////////////////////////////////////////////////
// Argument Parsing
//
// This section contains tests specific to argument parsing,
// including testing required arguments, as well as optional
// flag-specific checking.

// parse_show_no_args ensures show parses with no
// arguments (all DPAs).
#[test]
fn parse_show_no_args() {
    let cmd = Cmd::try_parse_from(["dpa", "show"]).expect("should parse show");

    match cmd {
        Cmd::Show(args) => {
            assert!(args.id.is_none());
        }
        other => panic!("expected Show, got {other:?}"),
    }
}

#[test]
fn parse_ensure_args() {
    let cmd = Cmd::try_parse_from([
        "dpa",
        "ensure",
        "fm100htes3rn1npvbtm5qd57dkilaag7ljugl1llmm7rfuq1ov50i0rpl30",
        "00:11:22:33:44:55",
        "BlueField3",
        "01:00.0",
    ])
    .expect("should parse ensure");

    match cmd {
        Cmd::Ensure(args) => {
            assert_eq!(
                args.machine_id.to_string(),
                "fm100htes3rn1npvbtm5qd57dkilaag7ljugl1llmm7rfuq1ov50i0rpl30"
            );
            assert_eq!(args.mac_addr, "00:11:22:33:44:55");
            assert_eq!(args.device_type, "BlueField3");
            assert_eq!(args.pci_name, "01:00.0");
        }
        other => panic!("expected Ensure, got {other:?}"),
    }
}
