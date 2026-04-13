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

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tracing::info;

pub async fn run(cache_dir: PathBuf, port: u16) -> Result<(), Box<dyn std::error::Error + Send>> {
    let app = Router::new().fallback_service(ServeDir::new(&cache_dir));
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;
    info!(%addr, dir = %cache_dir.display(), "File server listening");
    axum::serve(listener, app)
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;
    Ok(())
}
