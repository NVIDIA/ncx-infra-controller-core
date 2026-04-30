/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod common;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod forge;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod health;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod machine_discovery;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod measured_boot;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod mlx_device;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod site_explorer;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod dns;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod fmds;

#[allow(clippy::all, deprecated)]
#[rustfmt::skip]
pub mod forge_api_client;

#[allow(clippy::all)]
#[rustfmt::skip]
pub mod convenience_converters;

#[allow(non_snake_case, unknown_lints, clippy::all)]
#[rustfmt::skip]
pub mod dpa_rpc;

#[allow(clippy::all)]
#[rustfmt::skip]
pub mod nmx_c;

#[allow(clippy::all)]
#[rustfmt::skip]
pub mod nmx_c_client;

#[allow(clippy::all)]
#[rustfmt::skip]
pub mod nmx_c_converters;
