/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use libmlx::firmware::config::FirmwareFlasherProfile;
use libmlx::profile::serialization::SerializableProfile;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum OpCode<'a> {
    Noop,
    Unlock {
        key: String,
    },
    ApplyProfile {
        serialized_profile: Option<SerializableProfile>,
    },
    Lock {
        key: String,
    },
    ApplyFirmware {
        profile: Option<Box<Cow<'a, FirmwareFlasherProfile>>>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DpaCommand<'a> {
    pub op: OpCode<'a>,
}
