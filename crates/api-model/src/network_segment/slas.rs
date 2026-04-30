/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! SLAs for Network Segment State Machine Controller

use std::time::Duration;

pub const PROVISIONING: Duration = Duration::from_secs(15 * 60);
pub const DELETING_DBDELETE: Duration = Duration::from_secs(15 * 60);
