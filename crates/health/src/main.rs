/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_health::{Config, HealthError};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<(), HealthError> {
    let config_path = std::env::args().nth(1).map(std::path::PathBuf::from);
    let config = Config::load(config_path.as_deref()).map_err(HealthError::GenericError)?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(logfmt::layer())
        .with(env_filter)
        .init();

    tracing::info!(
        version = carbide_version::v!(build_version),
        config = ?config,
        "Started carbide-hw-health"
    );

    carbide_health::run_service(config).await?;

    tracing::info!(
        version = carbide_version::v!(build_version),
        "Stopped carbide-hw-health"
    );

    Ok(())
}
