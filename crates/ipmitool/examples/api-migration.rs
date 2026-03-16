/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Example showing how `carbide-api/src/ipmitool.rs` can be migrated
//! from shelling out to `/usr/bin/ipmitool` to using the native
//! `ipmitool` crate.
//!
//! This file is a self-contained sketch meant to illustrate the
//! migration pattern. It is NOT a compilable example (it references
//! `carbide-api` types that are not available in the ipmitool crate's
//! dev-dependencies). Delete this file after the migration.
//!
//! # What the current code does
//!
//! `carbide-api/src/ipmitool.rs` defines an `IPMITool` trait with two
//! methods:
//!
//! - `bmc_cold_reset()` → shells out: `ipmitool -I lanplus -C 17 bmc reset cold`
//! - `restart()` → shells out: `ipmitool -I lanplus -C 17 chassis power reset`
//!    (with an optional `raw 0x32 0xA1 0x01` for DPU legacy boot)
//!
//! Both methods resolve credentials from a `CredentialReader`, construct
//! CLI arguments, and spawn `/usr/bin/ipmitool` via `TokioCmd`.
//!
//! # Migration strategy
//!
//! Replace the `execute_ipmitool_command` shell-out with a helper that
//! establishes an RMCP+ session using `LanplusTransport::connect()` and
//! calls the appropriate `cmd::*` function directly.

// ==============================================================================
// Before: carbide-api/src/ipmitool.rs (current code, abbreviated)
// ==============================================================================
//
//   impl IPMIToolImpl {
//       async fn execute_ipmitool_command(
//           &self,
//           command: &str,
//           bmc_ip: IpAddr,
//           credentials: &Credentials,
//       ) -> CmdResult<String> {
//           let (username, password) = match credentials {
//               Credentials::UsernamePassword { username, password } => (username, password),
//           };
//           let prefix_args: Vec<String> =
//               vec!["-H", bmc_ip.to_string().as_str(), "-U", username, "-E"]
//                   .into_iter()
//                   .map(str::to_owned)
//                   .collect();
//           let mut args = prefix_args.to_owned();
//           args.extend(command.split(' ').map(str::to_owned));
//           let cmd = TokioCmd::new("/usr/bin/ipmitool")
//               .args(&args)
//               .attempts(self.attempts);
//           cmd.env("IPMITOOL_PASSWORD", password).output().await
//       }
//   }

// ==============================================================================
// After: using the ipmitool crate directly
// ==============================================================================

// These functions are illustrative — they show the migration pattern
// but are not called from main().
#![allow(dead_code)]

use std::net::IpAddr;

use ipmitool::cmd::{chassis, mc, raw};
use ipmitool::transport::lanplus::LanplusTransport;
use ipmitool::transport::IpmiTransport;
use ipmitool::ConnectionConfig;

/// Connect to a BMC and return an active transport session.
///
/// This replaces the per-command shell-out pattern. The returned
/// transport can be reused for multiple commands within the same
/// session, avoiding the overhead of a full RAKP handshake per
/// operation.
async fn connect_to_bmc(
    bmc_ip: IpAddr,
    username: &str,
    password: &str,
) -> eyre::Result<LanplusTransport> {
    let config = ConnectionConfig {
        host: bmc_ip.to_string(),
        username: username.to_owned(),
        password: password.to_owned(),
        cipher_suite: 17,
        ..Default::default()
    };

    let transport = LanplusTransport::connect(config)
        .await
        .map_err(|e| eyre::eyre!("RMCP+ connection to {bmc_ip} failed: {e}"))?;

    Ok(transport)
}

/// Replacement for `bmc_cold_reset()`.
///
/// Before:
///   execute_ipmitool_command("-I lanplus -C 17 bmc reset cold", bmc_ip, &creds)
///
/// After:
///   mc::cold_reset(&mut transport)
async fn bmc_cold_reset(
    bmc_ip: IpAddr,
    username: &str,
    password: &str,
) -> eyre::Result<()> {
    let mut transport = connect_to_bmc(bmc_ip, username, password).await?;

    mc::cold_reset(&mut transport)
        .await
        .map_err(|e| eyre::eyre!("BMC cold reset failed: {e}"))?;

    // Best-effort session close — the BMC is resetting anyway.
    transport.close().await.ok();
    Ok(())
}

/// Replacement for `restart()`.
///
/// Before:
///   if legacy_boot {
///       execute_ipmitool_command("-I lanplus -C 17 raw 0x32 0xA1 0x01", ...)
///   }
///   execute_ipmitool_command("-I lanplus -C 17 chassis power reset", ...)
///
/// After:
///   if legacy_boot {
///       raw::raw_command(&mut transport, 0x32, 0xA1, &[0x01])
///   }
///   chassis::chassis_control(&mut transport, chassis::PowerCommand::Reset)
async fn restart(
    bmc_ip: IpAddr,
    username: &str,
    password: &str,
    legacy_boot: bool,
) -> eyre::Result<()> {
    let mut transport = connect_to_bmc(bmc_ip, username, password).await?;

    if legacy_boot {
        // The DPU legacy command: raw 0x32 0xA1 0x01
        // NetFn 0x32 = OEM, Cmd 0xA1, Data = [0x01]
        match raw::raw_command(&mut transport, 0x32, 0xA1, &[0x01]).await {
            Ok(_) => {
                transport.close().await.ok();
                return Ok(());
            }
            Err(e) => {
                tracing::warn!(error = %e, "DPU legacy reset failed, falling back to chassis power reset");
            }
        }
    }

    chassis::chassis_control(&mut transport, chassis::PowerCommand::Reset)
        .await
        .map_err(|e| eyre::eyre!("chassis power reset failed: {e}"))?;

    transport.close().await.ok();
    Ok(())
}

// ==============================================================================
// Putting it together: how IPMIToolImpl changes
// ==============================================================================
//
// The `IPMITool` trait stays the same — callers are unaffected. Only
// `IPMIToolImpl` changes internally:
//
//   #[async_trait]
//   impl IPMITool for IPMIToolImpl {
//       async fn bmc_cold_reset(
//           &self,
//           bmc_ip: IpAddr,
//           credential_key: &CredentialKey,
//       ) -> Result<(), eyre::Report> {
//           let credentials = self.resolve_credentials(credential_key).await?;
//           let (username, password) = credentials.as_username_password()?;
//
//           let mut transport = connect_to_bmc(bmc_ip, username, password).await?;
//           mc::cold_reset(&mut transport)
//               .await
//               .map_err(|e| eyre::eyre!("BMC cold reset at {bmc_ip} failed: {e}"))?;
//           transport.close().await.ok();
//           Ok(())
//       }
//
//       async fn restart(
//           &self,
//           machine_id: &MachineId,
//           bmc_ip: IpAddr,
//           legacy_boot: bool,
//           credential_key: &CredentialKey,
//       ) -> Result<(), eyre::Report> {
//           let credentials = self.resolve_credentials(credential_key).await?;
//           let (username, password) = credentials.as_username_password()?;
//
//           let mut transport = connect_to_bmc(bmc_ip, username, password).await?;
//
//           if legacy_boot {
//               match raw::raw_command(&mut transport, 0x32, 0xA1, &[0x01]).await {
//                   Ok(_) => {
//                       transport.close().await.ok();
//                       return Ok(());
//                   }
//                   Err(e) => {
//                       tracing::warn!(
//                           %machine_id, error = %e,
//                           "DPU legacy reset failed, trying chassis power reset"
//                       );
//                   }
//               }
//           }
//
//           chassis::chassis_control(&mut transport, chassis::PowerCommand::Reset)
//               .await
//               .map_err(|e| eyre::eyre!(
//                   "chassis power reset for machine {machine_id} at {bmc_ip} failed: {e}"
//               ))?;
//
//           transport.close().await.ok();
//           Ok(())
//       }
//   }

// ==============================================================================
// What gets deleted
// ==============================================================================
//
// - `execute_ipmitool_command()` — the entire shell-out helper
// - The three `&str` constants (IPMITOOL_COMMAND_ARGS, etc.)
// - The `utils::cmd::{CmdError, CmdResult, TokioCmd}` imports
// - The `/usr/bin/ipmitool` binary from container images
//
// What stays:
// - The `IPMITool` trait (public interface unchanged)
// - The `IPMIToolTestImpl` mock (unchanged)
// - Credential resolution logic (moved to a small helper)

// ==============================================================================
// Cargo.toml change needed in carbide-api
// ==============================================================================
//
// [dependencies]
// carbide-ipmitool = { path = "../ipmitool" }

fn main() {
    // This example is documentation-only and not meant to be run.
    // See the code comments above for the migration pattern.
    println!("See source code comments for the migration pattern.");
}
