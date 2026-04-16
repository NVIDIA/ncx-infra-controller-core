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

use std::env;
use std::path::PathBuf;
use std::time::Duration;

use tracing::info;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheMode {
    S3,
    Local,
}

#[derive(Clone, Debug)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub internal_api_url: String,
    pub forge_root_ca_path: String,
    pub client_cert_path: String,
    pub client_key_path: String,
    pub mode: CacheMode,
    pub s3: Option<S3Config>,
    pub cache_dir: Option<PathBuf>,
    pub port: Option<u16>,
    pub temp_dir: PathBuf,
    pub poll_interval: Duration,
    pub max_file_size: u64,
    pub cache_as_needed: bool,
    pub tenant_filter: Option<String>,
    pub url_base: String,
    pub download_read_timeout: Duration,
}

impl RuntimeConfig {
    pub fn from_env() -> Result<Self, String> {
        if env::var("IMAGECACHE_MODE").is_err() {
            info!(
                "IMAGECACHE_MODE not set - default to sleep forever for unconfigured environments."
            );
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1000000));
            }
        }
        let poll_secs: u64 = env::var("IMAGECACHE_POLL_INTERVAL_SECS")
            .unwrap_or_else(|_| "3600".to_string())
            .parse()
            .map_err(|_| "IMAGECACHE_POLL_INTERVAL_SECS is not a valid u64".to_string())?;

        let max_file_size: u64 = env::var("IMAGECACHE_MAX_FILE_SIZE_BYTES")
            .unwrap_or_else(|_| "53687091200".to_string()) // 50 GiB
            .parse()
            .map_err(|_| "IMAGECACHE_MAX_FILE_SIZE_BYTES is not a valid u64".to_string())?;

        let cache_as_needed: bool = env::var("IMAGECACHE_CACHE_AS_NEEDED")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .map_err(|_| "IMAGECACHE_CACHE_AS_NEEDED is not a valid bool".to_string())?;

        let tenant_filter = env::var("IMAGECACHE_TENANT_FILTER").ok();

        let url_base = env::var("IMAGECACHE_URL_BASE")
            .map_err(|_| "Could not extract IMAGECACHE_URL_BASE from environment".to_string())?;
        let url_base = url_base.strip_suffix('/').unwrap_or(&url_base).to_string();

        let download_read_timeout_secs: u64 = env::var("IMAGECACHE_DOWNLOAD_READ_TIMEOUT_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .map_err(|_| "IMAGECACHE_DOWNLOAD_READ_TIMEOUT_SECS is not a valid u64".to_string())?;

        let mode = match env::var("IMAGECACHE_MODE")
            .unwrap_or_else(|_| "S3".to_string())
            .to_uppercase()
            .as_str()
        {
            "S3" => CacheMode::S3,
            "LOCAL" => CacheMode::Local,
            other => return Err(format!("IMAGECACHE_MODE must be S3 or LOCAL, got: {other}")),
        };

        let s3 = if mode == CacheMode::S3 {
            Some(S3Config {
                endpoint: env::var("IMAGECACHE_S3_ENDPOINT").map_err(|_| {
                    "Could not extract IMAGECACHE_S3_ENDPOINT from environment".to_string()
                })?,
                bucket: env::var("IMAGECACHE_S3_BUCKET").map_err(|_| {
                    "Could not extract IMAGECACHE_S3_BUCKET from environment".to_string()
                })?,
                region: env::var("IMAGECACHE_S3_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                access_key: env::var("IMAGECACHE_S3_ACCESS_KEY").map_err(|_| {
                    "Could not extract IMAGECACHE_S3_ACCESS_KEY from environment".to_string()
                })?,
                secret_key: env::var("IMAGECACHE_S3_SECRET_KEY").map_err(|_| {
                    "Could not extract IMAGECACHE_S3_SECRET_KEY from environment".to_string()
                })?,
            })
        } else {
            None
        };

        let cache_dir = if mode == CacheMode::Local {
            Some(PathBuf::from(env::var("IMAGECACHE_CACHE_DIR").map_err(
                |_| "IMAGECACHE_CACHE_DIR is required when IMAGECACHE_MODE=LOCAL".to_string(),
            )?))
        } else {
            None
        };

        let port = if mode == CacheMode::Local {
            Some(
                env::var("IMAGECACHE_PORT")
                    .map_err(|_| {
                        "IMAGECACHE_PORT is required when IMAGECACHE_MODE=LOCAL".to_string()
                    })?
                    .parse::<u16>()
                    .map_err(|_| "IMAGECACHE_PORT is not a valid u16".to_string())?,
            )
        } else {
            None
        };

        let temp_dir = match env::var("IMAGECACHE_TEMP_DIR") {
            Ok(dir) => PathBuf::from(dir),
            Err(_) if mode == CacheMode::Local => {
                // Default to cache_dir so downloads land on the same filesystem,
                // enabling atomic rename instead of a full copy.
                cache_dir.clone().expect("cache_dir already validated")
            }
            Err(_) => std::env::temp_dir(),
        };

        Ok(Self {
            internal_api_url: env::var("CARBIDE_API_INTERNAL_URL").unwrap_or_else(|_| {
                "https://carbide-api.forge-system.svc.cluster.local:1079".to_string()
            }),
            forge_root_ca_path: env::var("FORGE_ROOT_CAFILE_PATH").map_err(|_| {
                "Could not extract FORGE_ROOT_CAFILE_PATH from environment".to_string()
            })?,
            client_cert_path: env::var("FORGE_CLIENT_CERT_PATH").map_err(|_| {
                "Could not extract FORGE_CLIENT_CERT_PATH from environment".to_string()
            })?,
            client_key_path: env::var("FORGE_CLIENT_KEY_PATH").map_err(|_| {
                "Could not extract FORGE_CLIENT_KEY_PATH from environment".to_string()
            })?,
            mode,
            s3,
            cache_dir,
            port,
            temp_dir,
            poll_interval: Duration::from_secs(poll_secs),
            max_file_size,
            cache_as_needed,
            tenant_filter,
            url_base,
            download_read_timeout: Duration::from_secs(download_read_timeout_secs),
        })
    }
}
