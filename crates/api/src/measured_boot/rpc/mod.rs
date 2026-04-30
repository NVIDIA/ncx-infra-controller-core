/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/*!
 * The `rpc` module provides the gRPC layer to connect the Carbide
 * API to the underlying measured_boot code (model, interface, dto).
 *
 * This includes handlers for all gRPC calls, for all aspects of
 * measured boot:
 *  - `bundle`: Measurement bundles.
 *  - `journal`: Measurement journals.
 *  - `machine`: Mock machines (will eventually go away).
 *  - `profile`: System profiles.
 *  - `report`: Machine measurement reports.
 *  - `site`: Site management.
 */

pub mod bundle;
pub mod journal;
pub mod machine;
pub mod profile;
pub mod report;
pub mod site;
