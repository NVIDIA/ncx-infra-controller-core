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

//! Rack Validation Service (RVS)
//!
//! External validation orchestrator for BMMC. Bridges BMMC with test
//! frameworks (Benchpress, MPI-based, SLURM-based, etc.) to perform
//! partition-aware rack validation.
//!
//! NOTE: This is still a tracer / playground. The abstractions are
//! crystallizing but main.rs is not yet the final shape.

use std::error::Error;

use forge_tls::client_config::ClientCert;
use rpc::forge_tls_client::{ApiConfig, ForgeClientConfig};
use tokio::io::AsyncWriteExt;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod client;
mod config;
mod error;
mod partitions;
mod rack;
mod scenario;

use client::BmmcClient;
use client::convert::FetchedRack;
use config::Config;
use rack::Racks;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(logfmt::layer())
        .with(env_filter)
        .init();

    tracing::info!("carbide-rvs: Rack Validation Service starting");

    // Load config: defaults -> optional TOML -> CARBIDE_RVS__* env vars
    let cfg = Config::load(None)?;
    tracing::info!(config = ?cfg, "config loaded");

    // Try loading scenario -- soft fail, this is tracer code
    match scenario::Scenario::load(std::path::Path::new(&cfg.scenario_config_path)) {
        Ok(s) => tracing::info!(scenario = ?s, "scenario loaded"),
        Err(e) => tracing::warn!(error = %e, "scenario not loaded, continuing without it"),
    }

    // Build BMMC client from config
    let client_cert = ClientCert {
        cert_path: cfg.tls.identity_pemfile_path.clone(),
        key_path: cfg.tls.identity_keyfile_path.clone(),
    };
    let client_config = ForgeClientConfig::new(cfg.tls.root_cafile_path.clone(), Some(client_cert));
    let api_config = ApiConfig::new(&cfg.bmmc.url, &client_config);
    let bmmc = BmmcClient::new(&api_config);

    // Fetch racks and their machines via the client abstraction

    tracing::info!("tracer: fetching racks...");

    let rack_datas = bmmc.get_racks().await?;
    tracing::info!(count = rack_datas.len(), "tracer: got racks");

    let mut fetched_racks: Vec<FetchedRack> = Vec::new();

    for rack_data in rack_datas {
        tracing::info!(rack = ?rack_data, "tracer: fetching machines for rack");
        let trays = bmmc.get_machines(&rack_data).await?;
        tracing::info!(rack_id = %rack_data.id, trays = trays.len(), "tracer: got machines");
        fetched_racks.push(FetchedRack::new(rack_data.id, trays));
    }

    // Convert into the domain model

    tracing::info!("tracer: building domain model...");

    let racks = Racks::try_from(fetched_racks)?;

    tracing::info!(racks = ?racks, "tracer: === Racks model ===");

    // Liveness probe server

    let listen_addr = cfg.metrics_endpoint.to_string();
    tracing::info!(addr = %listen_addr, "starting liveness HTTP server");

    let listener = tokio::net::TcpListener::bind(cfg.metrics_endpoint).await?;

    loop {
        if let Ok((mut stream, _addr)) = listener.accept().await {
            tokio::spawn(async move {
                // TODO: proper responses instead of this
                let mut buf = [0u8; 1024];
                let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
                let body = "carbide-rvs: alive\n";
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    }
}
