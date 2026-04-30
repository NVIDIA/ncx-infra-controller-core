/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/*!
 * The `db` module provides the basic database CRUD logic for models in the `measured-boot` crate,
 * including:
 *
 *  - `bundle`: Measurement bundles.
 *  - `journal`: Measurement journals.
 *  - `machine`: Mock machines (will eventually go away).
 *  - `profile`: System profiles.
 *  - `report`: Machine measurement reports.
 *  - `site`: Site management.
 */

pub mod bundle;
pub mod interface;
pub mod journal;
pub mod machine;
pub mod profile;
pub mod report;
pub mod site;
