/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use health_report::{HealthReport, HealthReportApplyMode};
use sqlx::PgConnection;

use crate::DatabaseError;

/// Insert a health report into the `health_reports` JSONB column of the
/// given table.
///
/// The `id` parameter is bound as `$2` and must match the `id`
/// column of `table_name`.
pub async fn insert_health_report<Id>(
    txn: &mut PgConnection,
    table_name: &str,
    id: &Id,
    mode: HealthReportApplyMode,
    health_report: &HealthReport,
) -> Result<(), DatabaseError>
where
    for<'e> Id: sqlx::Encode<'e, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync,
{
    let column_name = "health_reports";
    let path = match mode {
        HealthReportApplyMode::Merge => format!("merges,\"{}\"", health_report.source),
        HealthReportApplyMode::Replace => "replace".to_string(),
    };

    let query = format!(
        "UPDATE {table_name} SET {column_name} = jsonb_set(
            coalesce({column_name}, '{{\"merges\": {{}}}}'::jsonb),
            '{{{path}}}',
            $1::jsonb
        ) WHERE id = $2
        RETURNING id"
    );

    sqlx::query(&query)
        .bind(sqlx::types::Json(health_report))
        .bind(id)
        .fetch_one(txn)
        .await
        .map_err(|e| DatabaseError::new(&format!("insert {table_name} health report"), e))?;

    Ok(())
}

/// Remove a health report from the `health_reports` JSONB column of the
/// given table.
pub async fn remove_health_report<Id>(
    txn: &mut PgConnection,
    table_name: &str,
    id: &Id,
    mode: HealthReportApplyMode,
    source: &str,
) -> Result<(), DatabaseError>
where
    for<'e> Id: sqlx::Encode<'e, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync,
{
    let column_name = "health_reports";
    let path = match mode {
        HealthReportApplyMode::Merge => format!("merges,{source}"),
        HealthReportApplyMode::Replace => "replace".to_string(),
    };
    let query = format!(
        "UPDATE {table_name} SET {column_name} = ({column_name} #- '{{{path}}}') WHERE id = $1
            RETURNING id"
    );

    sqlx::query(&query)
        .bind(id)
        .fetch_one(txn)
        .await
        .map_err(|e| DatabaseError::new(&format!("remove {table_name} health report"), e))?;

    Ok(())
}
