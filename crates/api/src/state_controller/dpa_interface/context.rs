/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::state_controller::common_services::CommonStateHandlerServices;
use crate::state_controller::dpa_interface::metrics::DpaInterfaceMetrics;
use crate::state_controller::state_handler::StateHandlerContextObjects;

pub struct DpaInterfaceStateHandlerContextObjects {}

impl StateHandlerContextObjects for DpaInterfaceStateHandlerContextObjects {
    type Services = CommonStateHandlerServices;
    type ObjectMetrics = DpaInterfaceMetrics;
}
