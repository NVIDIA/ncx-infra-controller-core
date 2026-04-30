/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimTableTarget {
    MeasuredBoot,
}

impl From<rpc::forge::TrimTableTarget> for TrimTableTarget {
    fn from(target: rpc::forge::TrimTableTarget) -> Self {
        match target {
            rpc::forge::TrimTableTarget::MeasuredBoot => TrimTableTarget::MeasuredBoot,
        }
    }
}
