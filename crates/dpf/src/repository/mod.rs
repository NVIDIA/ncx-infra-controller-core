/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Repository pattern for DPF Kubernetes resources.
//!
//! This module provides trait-based abstractions for interacting with DPF CRDs,
//! enabling dependency injection and testability.

mod kube;
mod traits;

pub use traits::*;

pub use self::kube::KubeRepository;
