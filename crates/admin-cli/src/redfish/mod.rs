/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod args;
pub mod cmds;

#[cfg(test)]
mod tests;

pub use args::{Cmd, RedfishAction, UriInfo};
pub use cmds::{action, handle_browse_command};
