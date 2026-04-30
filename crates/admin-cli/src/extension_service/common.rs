/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::ValueEnum;

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab_case")]
#[repr(i32)]
pub enum ExtensionServiceType {
    #[value(alias = "k8s")]
    KubernetesPod = 0, // Kubernetes pod service type
}

impl From<ExtensionServiceType> for i32 {
    fn from(v: ExtensionServiceType) -> Self {
        v as i32
    }
}
