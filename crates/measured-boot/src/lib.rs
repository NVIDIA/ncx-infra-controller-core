/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod bundle;
pub mod journal;
pub mod machine;
pub mod pcr;
pub mod profile;
pub mod records;
pub mod report;
pub mod site;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Parse(String),
    #[error("{0}")]
    RpcConversion(String),
}

pub type Result<T> = std::result::Result<T, Error>;
