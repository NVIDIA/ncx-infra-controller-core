/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// src/lib.rs
// Main exports for the mqttea MQTT client library.

pub mod auth;
pub mod client;
pub mod errors;
pub mod message_types;
pub mod registry;
pub mod stats;
pub mod traits;

// Export some things for convenience.
// Re-export auth types for convenience.
pub use auth::{
    ClientCredentialsProvider, ClientId, ClientSecret, CredentialsProvider, OAuth2Config,
    OAuth2TokenProvider, StaticCredentials, TokenCredentialsProvider, TokenProvider,
};
pub use client::{MqtteaClient, TopicPatterns};
pub use errors::MqtteaClientError;
pub use message_types::RawMessage;
pub use registry::{MessageTypeInfo, MqttRegistry, SerializationFormat};
pub use rumqttc::QoS;
pub use stats::{PublishStats, QueueStats};
pub use traits::{MessageHandler, MqttRecipient, RawMessageType};
