/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod dpu_extension_service_observability;
pub mod k8s_pod_handler;
pub mod manager;
pub mod service_handler;

pub use manager::ExtensionServiceManager;
