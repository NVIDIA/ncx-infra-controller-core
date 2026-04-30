/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use log::LevelFilter;

unsafe extern "C" {
    pub fn shim_version() -> libc::c_int;
    pub fn shim_load(_: *mut libc::c_void) -> libc::c_int;
    pub fn shim_unload() -> libc::c_int;
    pub fn shim_multi_threading_compatible() -> libc::c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn version() -> libc::c_int {
    unsafe { shim_version() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn load(a: *mut libc::c_void) -> libc::c_int {
    match log::set_logger(&crate::LOGGER).map(|()| log::set_max_level(LevelFilter::Trace)) {
        Ok(_) => log::info!("Initialized Logger"),
        Err(err) => {
            eprintln!("Unable to initialize logger: {err}");
            return 1;
        }
    };

    unsafe { shim_load(a) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn unload() -> libc::c_int {
    unsafe { shim_unload() }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn multi_threading_compatible() -> libc::c_int {
    unsafe { shim_multi_threading_compatible() }
}
