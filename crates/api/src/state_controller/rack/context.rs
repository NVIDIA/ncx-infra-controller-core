/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::state_controller::common_services::CommonStateHandlerServices;
use crate::state_controller::state_handler::StateHandlerContextObjects;

pub struct RackStateHandlerContextObjects {}

impl StateHandlerContextObjects for RackStateHandlerContextObjects {
    type ObjectMetrics = ();
    type Services = CommonStateHandlerServices;
}
