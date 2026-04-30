/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;

use crate::cfg::runtime::RuntimeContext;

// Run is a trait implemented by leaf argument structs,
// allowing them to execute themselves given a RuntimeContext.
// This complements Dispatch (which is implemented on the
// top-level Cmd enum) by pushing execution logic down to
// the individual command structs.
pub(crate) trait Run {
    fn run(
        self,
        ctx: &mut RuntimeContext,
    ) -> impl std::future::Future<Output = CarbideCliResult<()>>;
}
