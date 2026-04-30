/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// src/message_types/raw.rs
// Raw message types for binary data handling

use crate::traits::RawMessageType;

// RawMessage handles arbitrary binary data, including
// from unmapped MQTT topics.
#[derive(Clone, Debug, PartialEq)]
pub struct RawMessage {
    pub payload: Vec<u8>,
}

impl RawMessageType for RawMessage {
    fn to_bytes(&self) -> Vec<u8> {
        self.payload.clone()
    }

    fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { payload: bytes }
    }
}
