/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use chrono::{DateTime, Utc};
use config_version::ConfigVersion;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};

/// History of states for a single object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateHistoryRecord {
    /// The state that was entered
    pub state: String,
    // The version number associated with the state change
    pub state_version: ConfigVersion,
    /// The time when the state was observed.
    ///
    /// Older history payloads did not carry a separate timestamp, so RPC
    /// conversions fall back to the version timestamp when this field is absent.
    #[serde(default, alias = "timestamp")]
    pub time: Option<DateTime<Utc>>,
}

impl<'r> FromRow<'r, PgRow> for StateHistoryRecord {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let time = match row.try_get("timestamp") {
            Ok(time) => Some(time),
            Err(sqlx::Error::ColumnNotFound(_)) => match row.try_get("time") {
                Ok(time) => Some(time),
                Err(sqlx::Error::ColumnNotFound(_)) => None,
                Err(err) => return Err(err),
            },
            Err(err) => return Err(err),
        };

        Ok(StateHistoryRecord {
            state: row.try_get("state")?,
            state_version: row.try_get("state_version")?,
            time,
        })
    }
}

impl From<StateHistoryRecord> for ::rpc::forge::StateHistoryRecord {
    fn from(value: StateHistoryRecord) -> ::rpc::forge::StateHistoryRecord {
        let time = value
            .time
            .unwrap_or_else(|| value.state_version.timestamp());
        ::rpc::forge::StateHistoryRecord {
            state: value.state,
            version: value.state_version.version_string(),
            time: Some(time.into()),
        }
    }
}

impl From<StateHistoryRecord> for ::rpc::forge::MachineEvent {
    fn from(value: StateHistoryRecord) -> ::rpc::forge::MachineEvent {
        let time = value
            .time
            .unwrap_or_else(|| value.state_version.timestamp());
        ::rpc::forge::MachineEvent {
            event: value.state,
            version: value.state_version.version_string(),
            time: Some(time.into()),
        }
    }
}

impl From<StateHistoryRecord> for ::rpc::forge::NetworkSegmentStateHistory {
    fn from(value: StateHistoryRecord) -> ::rpc::forge::NetworkSegmentStateHistory {
        let time = value
            .time
            .unwrap_or_else(|| value.state_version.timestamp());
        ::rpc::forge::NetworkSegmentStateHistory {
            state: value.state,
            version: value.state_version.version_string(),
            time: Some(time.into()),
        }
    }
}
