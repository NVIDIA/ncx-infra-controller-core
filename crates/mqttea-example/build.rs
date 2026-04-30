/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// build.rs
// Builds the sample message protos.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::Config::new()
        .out_dir("src/sample_protos")
        .compile_protos(&["proto/messages.proto"], &["proto/"])?;
    println!("cargo:rerun-if-changed=proto/messages.proto");
    Ok(())
}
