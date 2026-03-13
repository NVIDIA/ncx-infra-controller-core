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

//! CLI subcommands for Serial-over-LAN (SOL) operations.
//!
//! Provides info, activate/deactivate, and configuration commands. The actual
//! bidirectional interactive terminal session is not yet implemented — it
//! requires transport-level SOL payload handling with async I/O.

use eyre::Context;

use crate::cmd::sol::{self, SolBitRate};
use crate::transport::IpmiTransport;

/// SOL subcommands.
#[derive(clap::Subcommand)]
pub enum SolCommand {
    /// Show SOL configuration for a channel.
    Info {
        /// IPMI channel number.
        #[arg(default_value = "1")]
        channel: u8,
    },
    /// Activate SOL session.
    ///
    /// NOTE: Full interactive SOL terminal is not yet implemented.
    /// This activates the SOL payload but does not handle bidirectional I/O.
    Activate {
        /// SOL payload instance number.
        #[arg(short, long, default_value = "1")]
        instance: u8,
    },
    /// Deactivate SOL session.
    Deactivate {
        /// SOL payload instance number.
        #[arg(short, long, default_value = "1")]
        instance: u8,
    },
    /// Set SOL baud rate.
    SetBaudRate {
        /// Baud rate: 9600, 19200, 38400, 57600, 115200.
        rate: String,
        /// IPMI channel number.
        #[arg(short, long, default_value = "1")]
        channel: u8,
        /// Set the volatile (session) rate instead of the non-volatile (persistent) rate.
        #[arg(long)]
        volatile: bool,
    },
    /// Enable SOL on a channel.
    Enable {
        /// IPMI channel number.
        #[arg(default_value = "1")]
        channel: u8,
    },
    /// Disable SOL on a channel.
    Disable {
        /// IPMI channel number.
        #[arg(default_value = "1")]
        channel: u8,
    },
}

/// Dispatch a SOL subcommand to the appropriate IPMI operations.
///
/// # Errors
///
/// Returns an error if the IPMI transport fails or the BMC returns an error
/// completion code.
pub async fn run(
    transport: &mut impl IpmiTransport,
    cmd: SolCommand,
) -> eyre::Result<()> {
    match cmd {
        SolCommand::Info { channel } => {
            let config = sol::get_sol_config(transport, channel)
                .await
                .context("get SOL configuration")?;

            println!("SOL Configuration (channel {channel}):");
            println!(
                "  Enabled            : {}",
                if config.enabled { "true" } else { "false" }
            );
            println!("  Privilege Level    : {}", config.privilege_level);
            println!("  Force Encryption   : {}", config.force_encryption);
            println!("  Force Authentication: {}", config.force_authentication);
            println!(
                "  Char Accumulate    : {} ms",
                config.char_accumulate_interval_ms
            );
            println!("  Char Send Threshold: {}", config.char_send_threshold);
            println!("  Retry Count        : {}", config.retry_count);
            println!("  Retry Interval     : {} ms", config.retry_interval_ms);
            println!(
                "  Non-volatile Rate  : {}",
                config.non_volatile_bit_rate
            );
            println!("  Volatile Rate      : {}", config.volatile_bit_rate);
            Ok(())
        }
        SolCommand::Activate { instance } => {
            let activation = sol::activate_sol(transport, instance, true, true)
                .await
                .context("activate SOL payload")?;

            println!("SOL payload activated (instance {instance}):");
            println!(
                "  Inbound Payload Size : {} bytes",
                activation.inbound_payload_size
            );
            println!(
                "  Outbound Payload Size: {} bytes",
                activation.outbound_payload_size
            );
            println!("  Port                 : {}", activation.port);
            println!(
                "  VLAN                 : {}",
                if activation.vlan == 0xFFFF {
                    "none".to_string()
                } else {
                    activation.vlan.to_string()
                }
            );

            // TODO: Implement bidirectional interactive SOL terminal with
            // transport-level SOL payload handling and CancellationToken for
            // graceful shutdown. This requires sending/receiving SOL payload
            // type 0x01 packets over the RMCP+ session, which is beyond
            // the current IPMI command-layer abstraction.
            println!();
            println!("Interactive SOL terminal not yet implemented. Use `sol deactivate` to clean up.");
            Ok(())
        }
        SolCommand::Deactivate { instance } => {
            sol::deactivate_sol(transport, instance)
                .await
                .context("deactivate SOL payload")?;

            println!("SOL payload deactivated (instance {instance})");
            Ok(())
        }
        SolCommand::SetBaudRate {
            rate,
            channel,
            volatile,
        } => {
            let bit_rate = parse_baud_rate(&rate)?;
            let label = if volatile {
                "volatile"
            } else {
                "non-volatile"
            };

            sol::set_sol_bit_rate(transport, channel, volatile, bit_rate)
                .await
                .context("set SOL bit rate")?;

            println!("Set {label} SOL baud rate to {bit_rate} on channel {channel}");
            Ok(())
        }
        SolCommand::Enable { channel } => {
            sol::set_sol_enable(transport, channel, true)
                .await
                .context("enable SOL")?;

            println!("SOL enabled on channel {channel}");
            Ok(())
        }
        SolCommand::Disable { channel } => {
            sol::set_sol_enable(transport, channel, false)
                .await
                .context("disable SOL")?;

            println!("SOL disabled on channel {channel}");
            Ok(())
        }
    }
}

/// Parse a baud rate string into a [`SolBitRate`].
fn parse_baud_rate(s: &str) -> eyre::Result<SolBitRate> {
    match s {
        "9600" => Ok(SolBitRate::Rate9600),
        "19200" => Ok(SolBitRate::Rate19200),
        "38400" => Ok(SolBitRate::Rate38400),
        "57600" => Ok(SolBitRate::Rate57600),
        "115200" => Ok(SolBitRate::Rate115200),
        "default" => Ok(SolBitRate::UseDefault),
        other => eyre::bail!(
            "unknown baud rate: {other} (supported: 9600, 19200, 38400, 57600, 115200, default)"
        ),
    }
}
