/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use chrono::Duration;
use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize_arc_atomic_bool<'de, D>(deserializer: D) -> Result<Arc<AtomicBool>, D::Error>
where
    D: Deserializer<'de>,
{
    let b = bool::deserialize(deserializer)?;
    Ok(Arc::new(b.into()))
}

pub fn serialize_arc_atomic_bool<S>(cm: &Arc<AtomicBool>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_bool(cm.load(AtomicOrdering::Relaxed))
}

/// As of now, chrono::Duration does not support Serialization, so we have to handle it manually.
pub fn as_duration<S>(d: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}s", d.num_seconds()))
}

pub fn as_std_duration<S>(d: &std::time::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}s", d.as_secs()))
}
