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

use ::rpc::admin_cli::{CarbideCliError, CarbideCliResult, OutputFormat};
use ::rpc::forge::ArtifactCacheStrategy;
use prettytable::{Cell, Row, Table};

use super::args::Args;
use crate::operating_system::common::{str_to_rpc_uuid, SerializableArtifact};
use crate::rpc::ApiClient;

pub async fn set_local_url(
    opts: Args,
    format: OutputFormat,
    api_client: &ApiClient,
) -> CarbideCliResult<()> {
    let id = str_to_rpc_uuid(&opts.id)?;

    let resp = api_client
        .0
        .set_operating_system_artifacts_local_url(
            ::rpc::forge::SetOperatingSystemArtifactsLocalUrlRequest {
                id: Some(id),
                updates: opts.updates,
            },
        )
        .await
        .map_err(CarbideCliError::from)?;

    if format == OutputFormat::Json {
        let serializable: Vec<SerializableArtifact> =
            resp.artifacts.into_iter().map(SerializableArtifact::from).collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serializable).map_err(CarbideCliError::JsonError)?
        );
        return Ok(());
    }

    println!("Updated artifacts for OS {}:", opts.id);

    let mut table = Table::new();
    table.set_titles(Row::new(vec![
        Cell::new("Name"),
        Cell::new("URL"),
        Cell::new("Local URL"),
        Cell::new("SHA"),
        Cell::new("Cache Strategy"),
    ]));

    for a in &resp.artifacts {
        let cache = match ArtifactCacheStrategy::try_from(a.cache_strategy) {
            Ok(ArtifactCacheStrategy::CacheAsNeeded) => "cache_as_needed",
            Ok(ArtifactCacheStrategy::LocalOnly) => "local_only",
            Ok(ArtifactCacheStrategy::CachedOnly) => "cached_only",
            Ok(ArtifactCacheStrategy::RemoteOnly) => "remote_only",
            _ => "unknown",
        };
        table.add_row(Row::new(vec![
            Cell::new(&a.name),
            Cell::new(&a.url),
            Cell::new(a.local_url.as_deref().unwrap_or("-")),
            Cell::new(a.sha.as_deref().unwrap_or("-")),
            Cell::new(cache),
        ]));
    }

    table.printstd();
    Ok(())
}
