/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/*!
 * The `interface` module provides thin functions to connect db
 * types to the database via queries.
 *
 * This includes basic insert/select/delete/update calls for:
 *  - `bundle`: Measurement bundles.
 *  - `common`: Generic functions leveraged by all interfaces.
 *  - `journal`: Measurement journals.
 *  - `machine`: Mock machines (will eventually go away).
 *  - `profile`: System profiles.
 *  - `report`: Machine measurement reports.
 *  - `site`: Site management.
 */

pub mod bundle;
pub mod common;
pub mod journal;
pub mod machine;
pub mod profile;
pub mod report;
pub mod site;
