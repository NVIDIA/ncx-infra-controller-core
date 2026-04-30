/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use model::site_explorer::SiteExplorationReport;

use crate::DatabaseError;
use crate::db_read::DbReader;

/// Fetches the latest site exploration report from the database
pub async fn fetch<DB>(db: &mut DB) -> Result<SiteExplorationReport, DatabaseError>
where
    for<'db> &'db mut DB: DbReader<'db>,
{
    let endpoints = crate::explored_endpoints::find_all(&mut *db).await?;
    let managed_hosts = crate::explored_managed_host::find_all(db).await?;
    Ok(SiteExplorationReport {
        endpoints,
        managed_hosts,
    })
}
