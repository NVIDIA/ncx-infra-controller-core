/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    carbide_version::build();

    // vendored from opentelemetry-proto v1.5.0
    let proto_dir = PathBuf::from("proto");

    println!("cargo:rerun-if-changed=proto/");

    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[proto_dir.join("opentelemetry/proto/collector/logs/v1/logs_service.proto")],
            &[proto_dir],
        )?;

    Ok(())
}
