/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[cfg(test)]
mod tests;

pub mod config;

pub mod downloader;

pub use config::{FirmwareConfig, FirmwareConfigSnapshot};
pub use downloader::FirmwareDownloader;
