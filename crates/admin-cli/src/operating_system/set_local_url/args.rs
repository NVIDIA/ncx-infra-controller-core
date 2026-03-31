/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use clap::Parser;

use ::rpc::forge::ArtifactLocalUrlUpdate;

fn parse_local_url_update(s: &str) -> Result<ArtifactLocalUrlUpdate, String> {
    let (name, url) = s
        .split_once('=')
        .ok_or_else(|| format!("expected NAME=URL (or NAME= to clear), got '{s}'"))?;
    let local_url = if url.is_empty() { None } else { Some(url.to_string()) };
    Ok(ArtifactLocalUrlUpdate {
        name: name.to_string(),
        local_url,
    })
}

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "UUID of the operating system definition.")]
    pub id: String,

    #[clap(
        long = "set",
        value_name = "NAME=URL",
        value_parser = parse_local_url_update,
        required = true,
        help = "Set local_url for an artifact. Use NAME=URL to set, NAME= to clear. May be repeated."
    )]
    pub updates: Vec<ArtifactLocalUrlUpdate>,
}
