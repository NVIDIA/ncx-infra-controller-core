/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// src/message_types/mod.rs
//
// First-class message types beyond protobufs,
// JSON, and YAML. Right now it's just the "raw"
// type, oh and the "string" type now too!

pub mod raw;
pub mod string;
pub use raw::RawMessage;
pub use string::StringMessage;
