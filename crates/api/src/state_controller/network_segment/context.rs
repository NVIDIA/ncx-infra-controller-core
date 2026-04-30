/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::state_controller::common_services::CommonStateHandlerServices;
use crate::state_controller::network_segment::metrics::NetworkSegmentMetrics;
use crate::state_controller::state_handler::StateHandlerContextObjects;

pub struct NetworkSegmentStateHandlerContextObjects {}

impl StateHandlerContextObjects for NetworkSegmentStateHandlerContextObjects {
    type Services = CommonStateHandlerServices;
    type ObjectMetrics = NetworkSegmentMetrics;
}
