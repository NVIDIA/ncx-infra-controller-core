/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use sqlx::PgPool;

/// This is re-used for every unit test as well as the migrate function. Do not call `sqlx::migrate!`
/// from anywhere else in the codebase, as it causes the migrations to be dumped into the binary
/// multiple times.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

#[tracing::instrument(skip(pool))]
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}
