/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

fn main() -> Result<(), Box<dyn std::error::Error>> {
    carbide_version::build();
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &["../dhcp-server/proto/dhcp_server_control.proto"],
            &["../dhcp-server/proto"],
        )?;
    Ok(())
}
