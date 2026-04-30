/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::ffi::OsStr;

use carbide_utils::cmd::TokioCmd;
use scout::CarbideClientError;

pub async fn run_prog<I, S>(command: S, args: I) -> Result<String, CarbideClientError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let command = TokioCmd::new(command);
    command
        .args(args)
        .output()
        .await
        .map_err(CarbideClientError::from)
}
