/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fs;
use std::path::Path;

use ::rpc::admin_cli::CarbideCliResult;

use super::super::add::cmd::add_individual;
use crate::rpc::ApiClient;

pub async fn add_bulk(dirname: &str, api_client: &ApiClient) -> CarbideCliResult<()> {
    let dirpath = Path::new(dirname);

    // read all files ending with .cer/.der
    // call add individually for each one of them

    let dir_entry_iter = fs::read_dir(dirpath)
        .map_err(::rpc::admin_cli::CarbideCliError::IOError)?
        .flatten();

    for dir_entry in dir_entry_iter {
        if (dir_entry.path().with_extension("cer").is_file()
            || dir_entry.path().with_extension("der").is_file())
            && let Err(e) = add_individual(dir_entry.path().as_path(), false, api_client).await
        {
            // we log the error but continue the iteration
            eprintln!("Could not add ca cert {dir_entry:?}: {e}");
        }
        if dir_entry.path().with_extension("pem").is_file()
            && let Err(e) = add_individual(dir_entry.path().as_path(), true, api_client).await
        {
            // we log the error but continue the iteration
            eprintln!("Could not add ca cert {dir_entry:?}: {e}");
        }
    }

    Ok(())
}
