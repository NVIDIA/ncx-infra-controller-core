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

use std::path::{Path, PathBuf};

use reqwest::Client;
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;

use crate::error::ImageCacheError;

pub struct DownloadResult {
    pub temp_path: PathBuf,
    pub sha256: String,
    _temp_guard: tempfile::TempPath,
}

pub async fn check_remote_size(
    client: &Client,
    url: &str,
    auth_type: Option<&str>,
    auth_token: Option<&str>,
    max_size: u64,
) -> Result<(), ImageCacheError> {
    let mut request = client.head(url);
    request = apply_auth(request, auth_type, auth_token);

    let response = request.send().await?.error_for_status()?;

    if let Some(content_length) = response.content_length()
        && content_length > max_size
    {
        return Err(ImageCacheError::TooLarge {
            size: content_length,
            limit: max_size,
        });
    }
    Ok(())
}

pub async fn download_and_hash(
    client: &Client,
    url: &str,
    auth_type: Option<&str>,
    auth_token: Option<&str>,
    expected_sha: Option<&str>,
    max_size: u64,
    temp_dir: &Path,
) -> Result<DownloadResult, ImageCacheError> {
    check_remote_size(client, url, auth_type, auth_token, max_size).await?;

    let mut request = client.get(url);
    request = apply_auth(request, auth_type, auth_token);

    let mut response = request.send().await?.error_for_status()?;

    let named = NamedTempFile::new_in(temp_dir)?;
    let temp_path = named.path().to_path_buf();
    let (std_file, temp_guard) = named.into_parts();
    let mut file = tokio::fs::File::from_std(std_file);
    let mut hasher = Sha256::new();
    let mut bytes_written: u64 = 0;

    while let Some(chunk) = response.chunk().await? {
        bytes_written += chunk.len() as u64;
        if bytes_written > max_size {
            return Err(ImageCacheError::TooLarge {
                size: bytes_written,
                limit: max_size,
            });
        }
        hasher.update(&chunk);
        file.write_all(&chunk).await?;
    }
    file.flush().await?;
    drop(file);

    let sha256 = hex::encode(hasher.finalize());

    if let Some(expected) = expected_sha
        && sha256 != expected.to_ascii_lowercase()
    {
        return Err(ImageCacheError::ChecksumMismatch {
            expected: expected.to_string(),
            actual: sha256,
        });
    }

    Ok(DownloadResult {
        temp_path,
        sha256,
        _temp_guard: temp_guard,
    })
}

fn apply_auth(
    request: reqwest::RequestBuilder,
    auth_type: Option<&str>,
    auth_token: Option<&str>,
) -> reqwest::RequestBuilder {
    match (auth_type, auth_token) {
        (Some("Bearer"), Some(token)) => request.bearer_auth(token),
        (Some("Basic"), Some(token)) => request.header("Authorization", format!("Basic {token}")),
        _ => request,
    }
}
