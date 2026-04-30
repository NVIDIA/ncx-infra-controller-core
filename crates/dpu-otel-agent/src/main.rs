/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::time::Duration;

use tracing_subscriber::fmt;

fn main() -> eyre::Result<()> {
    fmt().init();
    // We need a multi-threaded runtime since background threads will queue work
    // on it, and the foreground thread might not be blocked onto the runtime
    // at all points in time
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(otel_agent::start(otel_agent::Options::load()))?;
    rt.shutdown_timeout(Duration::from_secs(2));
    Ok(())
}
