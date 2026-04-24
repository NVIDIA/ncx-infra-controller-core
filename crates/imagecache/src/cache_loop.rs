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

use std::collections::HashSet;

use carbide_uuid::operating_system::OperatingSystemId;
use reqwest::Client;
use rpc::forge::IpxeTemplateArtifactUpdateRequest;
use tracing::{error, info, warn};

use crate::api_client::ApiClient;
use crate::config::RuntimeConfig;
use crate::error::ImageCacheError;
use crate::storage::StorageBackend;
use crate::{artifact, download};

struct CacheContext<'a, S: StorageBackend> {
    api: &'a ApiClient,
    http: &'a Client,
    storage: &'a S,
    config: &'a RuntimeConfig,
    uploaded_this_cycle: &'a mut HashSet<String>,
}

pub async fn run_once<S: StorageBackend>(
    api: &ApiClient,
    http: &Client,
    storage: &S,
    config: &RuntimeConfig,
) -> Result<(), ImageCacheError> {
    info!("Starting cache cycle");

    let os_ids = api.discover_os_ids(config.tenant_filter.clone()).await?;
    info!(
        count = os_ids.len(),
        "Discovered operating system definitions"
    );

    if os_ids.is_empty() {
        return Ok(());
    }

    let os_definitions = api.get_os_definitions(os_ids).await?;

    let mut cached_count: u32 = 0;
    let mut skipped_count: u32 = 0;
    let mut error_count: u32 = 0;
    let mut uploaded_this_cycle: HashSet<String> = HashSet::new();

    let mut ctx = CacheContext {
        api,
        http,
        storage,
        config,
        uploaded_this_cycle: &mut uploaded_this_cycle,
    };

    for os_def in &os_definitions {
        let os_id = match &os_def.id {
            Some(id) => *id,
            None => continue,
        };
        let os_name = &os_def.name;

        for art in &os_def.ipxe_template_artifacts {
            if !artifact::is_eligible(art.cache_strategy(), config.cache_as_needed) {
                skipped_count += 1;
                continue;
            }

            match process_artifact(&mut ctx, &os_id, os_name, art).await {
                Ok(true) => cached_count += 1,
                Ok(false) => skipped_count += 1,
                Err(e) => {
                    error!(
                        os = os_name,
                        artifact = art.name,
                        error = %e,
                        "Failed to process artifact"
                    );
                    error_count += 1;
                }
            }
        }
    }

    info!(
        cached = cached_count,
        skipped = skipped_count,
        errors = error_count,
        "Cache cycle complete"
    );
    Ok(())
}

/// Builds the S3 object key from a SHA256 hash, preserving the file extension
/// from the original URL. For example, if the URL ends in `.iso` and the hash
/// is `abc123`, the key becomes `abc123.iso`.
fn s3_key_for_artifact(sha: &str, url: &str) -> String {
    let ext = extension_from_url(url);
    let sha_lower = sha.to_ascii_lowercase();
    format!("{sha_lower}{ext}")
}

/// Extracts the file extension (including the leading dot) from a URL path.
/// Returns an empty string when there is no extension.
fn extension_from_url(url: &str) -> &str {
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);
    let filename = path.rsplit('/').next().unwrap_or(path);
    match filename.rfind('.') {
        Some(pos) if pos > 0 => &filename[pos..],
        _ => "",
    }
}

async fn process_artifact<S: StorageBackend>(
    ctx: &mut CacheContext<'_, S>,
    os_id: &OperatingSystemId,
    os_name: &str,
    art: &rpc::forge::IpxeTemplateArtifact,
) -> Result<bool, ImageCacheError> {
    // If the artifact already has a cached_url, verify it still exists in storage
    // and that its key matches the current SHA (detects upstream content changes).
    if let Some(cached_url) = &art.cached_url
        && !cached_url.is_empty()
    {
        let cached_key = cached_url.rsplit('/').next().unwrap_or(cached_url);

        let sha_matches = match &art.sha {
            Some(sha) if !sha.is_empty() => cached_key == s3_key_for_artifact(sha, &art.url),
            _ => true, // No SHA to compare against
        };

        if !sha_matches {
            warn!(
                os = os_name,
                artifact = art.name,
                "Cached artifact does not match current SHA, clearing cached_url"
            );
            ctx.api
                .set_artifact_cached_urls(
                    *os_id,
                    vec![IpxeTemplateArtifactUpdateRequest {
                        name: art.name.clone(),
                        cached_url: None,
                    }],
                )
                .await?;
        } else {
            match ctx.storage.object_exists(cached_key).await {
                Ok(true) => {
                    return Ok(false); // Already cached
                }
                Ok(false) => {
                    warn!(
                        os = os_name,
                        artifact = art.name,
                        "Cached artifact missing from storage, clearing cached_url"
                    );
                    ctx.api
                        .set_artifact_cached_urls(
                            *os_id,
                            vec![IpxeTemplateArtifactUpdateRequest {
                                name: art.name.clone(),
                                cached_url: None,
                            }],
                        )
                        .await?;
                }
                Err(e) => {
                    warn!(
                        os = os_name,
                        artifact = art.name,
                        error = %e,
                        "Failed to HEAD-check cached_url, will re-download"
                    );
                }
            }
        }
    }

    // If we have a sha, check if the object already exists in storage (may have been
    // stored for another OS definition that shares the same artifact)
    if let Some(sha) = &art.sha
        && !sha.is_empty()
    {
        let key = s3_key_for_artifact(sha, &art.url);
        if ctx.storage.object_exists(&key).await.unwrap_or(false)
            || ctx.uploaded_this_cycle.contains(&key)
        {
            let cached_url = format!("{}/{key}", ctx.config.url_base);
            info!(
                os = os_name,
                artifact = art.name,
                "Artifact already in storage, setting cached_url"
            );
            ctx.api
                .set_artifact_cached_urls(
                    *os_id,
                    vec![IpxeTemplateArtifactUpdateRequest {
                        name: art.name.clone(),
                        cached_url: Some(cached_url),
                    }],
                )
                .await?;
            return Ok(true);
        }
    }

    // Download the artifact
    info!(
        os = os_name,
        artifact = art.name,
        url = art.url,
        "Downloading artifact"
    );

    let result = download::download_and_hash(
        ctx.http,
        &art.url,
        art.auth_type.as_deref(),
        art.auth_token.as_deref(),
        art.sha.as_deref(),
        ctx.config.max_file_size,
        &ctx.config.temp_dir,
    )
    .await?;

    // Store using the SHA256 (plus original file extension) as the key
    let key = s3_key_for_artifact(&result.sha256, &art.url);

    if !ctx.storage.object_exists(&key).await.unwrap_or(false) {
        info!(
            os = os_name,
            artifact = art.name,
            sha256 = %key,
            "Storing artifact"
        );
        ctx.storage
            .put_object_from_file(&key, &result.temp_path)
            .await?;
    }

    ctx.uploaded_this_cycle.insert(key.clone());

    // Update the local URL in the API
    let cached_url = format!("{}/{key}", ctx.config.url_base);
    ctx.api
        .set_artifact_cached_urls(
            *os_id,
            vec![IpxeTemplateArtifactUpdateRequest {
                name: art.name.clone(),
                cached_url: Some(cached_url),
            }],
        )
        .await?;

    info!(
        os = os_name,
        artifact = art.name,
        sha256 = %key,
        "Artifact cached successfully"
    );
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_from_url_iso() {
        assert_eq!(extension_from_url("https://example.com/ubuntu.iso"), ".iso");
    }

    #[test]
    fn extension_from_url_with_query_string() {
        assert_eq!(
            extension_from_url("https://example.com/image.qcow2?token=abc"),
            ".qcow2"
        );
    }

    #[test]
    fn extension_from_url_no_extension() {
        assert_eq!(extension_from_url("https://example.com/firmware"), "");
    }

    #[test]
    fn extension_from_url_hidden_file() {
        // A dotfile like ".hidden" has no meaningful extension for our purposes
        assert_eq!(extension_from_url("https://example.com/.hidden"), "");
    }

    #[test]
    fn extension_from_url_with_fragment() {
        assert_eq!(
            extension_from_url("https://example.com/boot.img#section"),
            ".img"
        );
    }

    #[test]
    fn s3_key_appends_extension() {
        let key = s3_key_for_artifact("abc123", "https://example.com/ubuntu.iso");
        assert_eq!(key, "abc123.iso");
    }

    #[test]
    fn s3_key_no_extension() {
        let key = s3_key_for_artifact("abc123", "https://example.com/firmware");
        assert_eq!(key, "abc123");
    }

    #[test]
    fn s3_key_normalizes_sha_to_lowercase() {
        let key = s3_key_for_artifact("ABC123", "https://example.com/ubuntu.iso");
        assert_eq!(key, "abc123.iso");
    }

    #[test]
    fn s3_key_mixed_case_sha_matches_lowercase() {
        let upper = s3_key_for_artifact("AbCdEf", "https://example.com/boot.img");
        let lower = s3_key_for_artifact("abcdef", "https://example.com/boot.img");
        assert_eq!(upper, lower);
    }
}
