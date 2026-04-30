/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::time::Duration;

use carbide_firmware::FirmwareConfig;

#[derive(Clone)]
pub struct PreingestionManagerConfig {
    pub run_interval: Duration,
    pub concurrency_limit: usize,
    pub hgx_bmc_gpu_reboot_delay: Duration,
    pub max_concurrent_bfb_copies: usize,
    pub autoupdate: bool,
    pub no_reset_retries: bool,
    pub firmware: FirmwareConfig,
}
