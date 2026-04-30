/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fmt;

use nv_redfish::{Bmc, Error as NvRedfishError};

pub enum Error<B: Bmc> {
    NvRedfish {
        context: &'static str,
        err: NvRedfishError<B>,
    },
    BmcNotProvided(&'static str),
    InvalidValue(String),
}

impl<B: Bmc> fmt::Display for Error<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NvRedfish { context, err } => write!(f, "redfish error in {context}: {err}"),
            Self::BmcNotProvided(what) => write!(f, "BMC has not provided {what}"),
            Self::InvalidValue(what) => write!(f, "Invalid value {what}"),
        }
    }
}

// Need to implement Debug manually because Derive requires B to
// implement Debug.
impl<B: Bmc> fmt::Debug for Error<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NvRedfish { context, err } => f
                .debug_struct("NvRedfish")
                .field("context", context)
                .field("err", &format_args!("{err}"))
                .finish(),
            Self::BmcNotProvided(what) => f.debug_tuple("BmcNotProvided").field(what).finish(),
            Self::InvalidValue(what) => f.debug_tuple("InvalidValue").field(what).finish(),
        }
    }
}

impl<B: Bmc> Error<B> {
    pub(crate) fn nv_redfish(context: &'static str) -> impl Fn(NvRedfishError<B>) -> Self {
        move |err| Self::NvRedfish { context, err }
    }

    pub(crate) fn bmc_not_provided(what: &'static str) -> impl Fn() -> Self {
        move || Self::BmcNotProvided(what)
    }
}
