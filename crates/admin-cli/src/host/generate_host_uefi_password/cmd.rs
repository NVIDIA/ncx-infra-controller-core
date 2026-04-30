/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use ::rpc::admin_cli::CarbideCliResult;
use forge_secrets::credentials::Credentials;

pub fn generate_uefi_password() -> CarbideCliResult<()> {
    let password = Credentials::generate_password_no_special_char();
    println!("Generated Bios Admin Password: {password}");
    Ok(())
}
