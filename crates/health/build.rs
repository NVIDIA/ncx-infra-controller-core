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

const OTLP_PROTO_VERSION: &str = "v1.5.0";
const OTLP_PROTO_BASE_URL: &str =
    "https://raw.githubusercontent.com/open-telemetry/opentelemetry-proto";

const OTLP_PROTO_FILES: &[&str] = &[
    "opentelemetry/proto/common/v1/common.proto",
    "opentelemetry/proto/resource/v1/resource.proto",
    "opentelemetry/proto/logs/v1/logs.proto",
    "opentelemetry/proto/collector/logs/v1/logs_service.proto",
];

// production CI/Docker builds should set OTLP_PROTO_DIR to a pre-fetched Dockerfile layer to avoid runtime network deps
// the reqwest fallback below should only be used for local development.
fn fetch_otlp_protos(out_dir: &Path) -> PathBuf {
    if let Ok(dir) = std::env::var("OTLP_PROTO_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return path;
        }
    }

    let proto_dir = out_dir.join("otlp-proto");

    for proto_file in OTLP_PROTO_FILES {
        let dest = proto_dir.join(proto_file);
        if dest.exists() {
            continue;
        }

        std::fs::create_dir_all(dest.parent().unwrap()).expect("create proto parent dirs");

        let url = format!("{OTLP_PROTO_BASE_URL}/{OTLP_PROTO_VERSION}/{proto_file}");
        let bytes = reqwest::blocking::get(&url)
            .and_then(|r| r.error_for_status())
            .unwrap_or_else(|e| panic!("failed to download {url}: {e}"))
            .bytes()
            .unwrap_or_else(|e| panic!("failed to read response body from {url}: {e}"));

        std::fs::write(&dest, &bytes)
            .unwrap_or_else(|e| panic!("failed to write {}: {e}", dest.display()));
    }

    proto_dir
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    carbide_version::build();

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let proto_dir = fetch_otlp_protos(&out_dir);

    println!("cargo:rerun-if-env-changed=OTLP_PROTO_DIR");

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[proto_dir.join("opentelemetry/proto/collector/logs/v1/logs_service.proto")],
            &[proto_dir],
        )?;

    Ok(())
}
