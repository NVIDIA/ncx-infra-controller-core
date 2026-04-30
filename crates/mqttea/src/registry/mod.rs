/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// src/registry/mod.rs
// Registry module coordination and re-exports for client-scoped
// message registration.

mod core;
mod entry;
pub mod traits;
pub mod types;

pub use core::MqttRegistry;

pub use entry::MqttRegistryEntry;
pub use traits::{
    JsonRegistration, MessageRegistration, ProtobufRegistration, RawRegistration, YamlRegistration,
};
pub use types::{DeserializeHandler, MessageTypeInfo, SerializationFormat, SerializeHandler};
