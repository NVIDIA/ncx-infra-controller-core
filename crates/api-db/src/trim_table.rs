/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use sqlx::PgConnection;

use crate::DatabaseError;

pub async fn trim_table(
    txn: &mut PgConnection,
    target: model::trim_table::TrimTableTarget,
    keep_entries: u32,
) -> Result<i32, DatabaseError> {
    // choose a target and call an appropriate stored procedure/function
    match target {
        model::trim_table::TrimTableTarget::MeasuredBoot => {
            let query = "SELECT * FROM measured_boot_reports_keep_limit($1)";

            let val: (i32,) = sqlx::query_as(query)
                .bind(keep_entries as i32)
                .fetch_one(txn)
                .await
                .map_err(|e| DatabaseError::new(query, e))?;
            Ok(val.0)
        }
    }
}
