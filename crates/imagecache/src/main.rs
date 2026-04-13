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

use std::time::Duration;

use clap::Parser;
use forge_tls::client_config::ClientCert;
use rpc::forge_api_client::ForgeApiClient;
use rpc::forge_tls_client::{ApiConfig, ForgeClientConfig};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod api_client;
mod artifact;
mod cache_loop;
mod config;
mod download;
mod error;
mod file_server;
mod local_storage;
mod s3;
mod storage;

use config::CacheMode;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value = "false", help = "Print version number and exit")]
    pub version: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Args::parse();
    if opts.version {
        println!("{}", carbide_version::version!());
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!(
        version = carbide_version::version!(),
        "Starting carbide-imagecache"
    );

    let runtime_config = config::RuntimeConfig::from_env().expect("unable to build runtime config");

    let client_cert = Some(ClientCert {
        cert_path: runtime_config.client_cert_path.clone(),
        key_path: runtime_config.client_key_path.clone(),
    });
    let client_config =
        ForgeClientConfig::new(runtime_config.forge_root_ca_path.clone(), client_cert);

    let api = api_client::ApiClient(ForgeApiClient::new(&ApiConfig::new(
        &runtime_config.internal_api_url,
        &client_config,
    )));

    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .read_timeout(runtime_config.download_read_timeout)
        .build()
        .expect("unable to build HTTP client");

    info!(
        poll_interval_secs = runtime_config.poll_interval.as_secs(),
        cache_as_needed = runtime_config.cache_as_needed,
        max_file_size = runtime_config.max_file_size,
        download_read_timeout_secs = runtime_config.download_read_timeout.as_secs(),
        temp_dir = %runtime_config.temp_dir.display(),
        mode = ?runtime_config.mode,
        "Configuration loaded"
    );

    tokio::fs::create_dir_all(&runtime_config.temp_dir).await?;

    match runtime_config.mode {
        CacheMode::S3 => {
            let s3_config = runtime_config.s3.as_ref().expect("S3 config required");
            let s3_client = s3::S3Client::new(s3_config).expect("unable to initialize S3 client");

            loop {
                if let Err(e) = cache_loop::run_once(&api, &http, &s3_client, &runtime_config).await
                {
                    error!(error = %e, "Cache cycle failed");
                }
                tokio::time::sleep(runtime_config.poll_interval).await;
            }
        }
        CacheMode::Local => {
            let cache_dir = runtime_config
                .cache_dir
                .as_ref()
                .expect("cache_dir required in LOCAL mode")
                .clone();
            let port = runtime_config.port.expect("port required in LOCAL mode");

            tokio::fs::create_dir_all(&cache_dir).await?;

            let local_storage = local_storage::LocalStorage::new(cache_dir.clone());

            let file_server = tokio::spawn(async move { file_server::run(cache_dir, port).await });

            let cache_loop = async {
                loop {
                    if let Err(e) =
                        cache_loop::run_once(&api, &http, &local_storage, &runtime_config).await
                    {
                        error!(error = %e, "Cache cycle failed");
                    }
                    tokio::time::sleep(runtime_config.poll_interval).await;
                }
            };

            tokio::select! {
                result = file_server => {
                    match result {
                        Ok(Ok(())) => error!("File server exited unexpectedly"),
                        Ok(Err(e)) => error!(error = %e, "File server exited with error"),
                        Err(e) => error!(error = %e, "File server task panicked"),
                    }
                    Err("file server exited unexpectedly".into())
                }
                _ = cache_loop => unreachable!(),
            }
        }
    }
}
