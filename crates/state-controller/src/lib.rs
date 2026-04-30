/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg_attr(test, allow(txn_held_across_await))]
#![cfg_attr(test, allow(txn_without_commit))]

#[cfg(any(test, feature = "test-support"))]
/// test_assert will run an assertion if we are compiled to run tests, or will print an error otherwise.
macro_rules! test_assert {
    ($cond:expr $(,)?) => {
        assert!($cond);
    };
    ($cond:expr, $($arg:tt)+) => {
        assert!($cond, $($arg)+);
    };
}

#[cfg(not(any(test, feature = "test-support")))]
/// test_assert will run an assertion if we are compiled to run tests, or will print an error otherwise.
macro_rules! test_assert {
    ($cond:expr $(,)?) => {
        if !$cond {
            tracing::error!(
                assertion = stringify!($cond),
                "test-only assertion failed"
            );
        }
    };
    ($cond:expr, $($arg:tt)+) => {
        if !$cond {
            tracing::error!($($arg)+);
        }
    };
}

pub mod config;
pub mod controller;
pub mod db_write_batch;
pub mod io;
pub mod metrics;
pub mod state_change_emitter;
pub mod state_handler;

#[cfg(test)]
mod tests;
