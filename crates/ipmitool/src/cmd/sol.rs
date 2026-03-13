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

//! Serial-over-LAN (SOL) commands: activate, deactivate, configuration.
//!
//! These implement the IPMI command-layer operations for SOL. The actual
//! bidirectional terminal session (SOL payload type 0x01 data transfer)
//! requires transport-level support and is not yet implemented.
//!
//! Activate/Deactivate use [`NetFn::App`] (0x06), while configuration
//! get/set use [`NetFn::Transport`] (0x0C).

use crate::error::{IpmitoolError, Result};
use crate::transport::IpmiTransport;
use crate::types::{IpmiRequest, NetFn};

// ==============================================================================
// SOL Types
// ==============================================================================

/// SOL baud rate selections (IPMI v2.0 Table 26-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolBitRate {
    /// Use the default rate from SOL configuration.
    UseDefault,
    Rate9600,
    Rate19200,
    Rate38400,
    Rate57600,
    Rate115200,
    /// A rate value not recognized by this implementation.
    Unknown(u8),
}

impl SolBitRate {
    /// Encode to the IPMI parameter byte value.
    fn as_byte(self) -> u8 {
        match self {
            Self::UseDefault => 0x00,
            Self::Rate9600 => 0x06,
            Self::Rate19200 => 0x07,
            Self::Rate38400 => 0x08,
            Self::Rate57600 => 0x09,
            Self::Rate115200 => 0x0A,
            Self::Unknown(v) => v,
        }
    }

    /// Decode from the IPMI parameter byte value.
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::UseDefault,
            0x06 => Self::Rate9600,
            0x07 => Self::Rate19200,
            0x08 => Self::Rate38400,
            0x09 => Self::Rate57600,
            0x0A => Self::Rate115200,
            other => Self::Unknown(other),
        }
    }
}

impl std::fmt::Display for SolBitRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UseDefault => write!(f, "use-default"),
            Self::Rate9600 => write!(f, "9600"),
            Self::Rate19200 => write!(f, "19200"),
            Self::Rate38400 => write!(f, "38400"),
            Self::Rate57600 => write!(f, "57600"),
            Self::Rate115200 => write!(f, "115200"),
            Self::Unknown(v) => write!(f, "unknown(0x{v:02X})"),
        }
    }
}

/// SOL configuration parameters read from the BMC.
#[derive(Debug, Clone)]
pub struct SolConfig {
    /// Whether SOL is enabled on this channel.
    pub enabled: bool,
    /// Minimum privilege level required for SOL activation (raw 4-bit value).
    pub privilege_level: u8,
    /// Whether the BMC forces encryption on SOL payloads.
    pub force_encryption: bool,
    /// Whether the BMC forces authentication on SOL payloads.
    pub force_authentication: bool,
    /// Character accumulate interval in 5ms increments.
    pub char_accumulate_interval_ms: u16,
    /// Number of characters the BMC will buffer before sending a packet.
    pub char_send_threshold: u8,
    /// Number of times the BMC will retry SOL packets.
    pub retry_count: u8,
    /// Retry interval in 10ms increments.
    pub retry_interval_ms: u16,
    /// Non-volatile (persistent) baud rate setting.
    pub non_volatile_bit_rate: SolBitRate,
    /// Volatile (session) baud rate setting.
    pub volatile_bit_rate: SolBitRate,
}

/// Response from activating a SOL payload on the BMC.
#[derive(Debug, Clone)]
pub struct SolActivation {
    /// Auxiliary data from the BMC (4 bytes, implementation-defined).
    pub aux_data: [u8; 4],
    /// Maximum payload size the BMC can receive (inbound to BMC).
    pub inbound_payload_size: u16,
    /// Maximum payload size the BMC will send (outbound from BMC).
    pub outbound_payload_size: u16,
    /// UDP port number for SOL payload traffic.
    pub port: u16,
    /// VLAN tag (0xFFFF if not used).
    pub vlan: u16,
}

// ==============================================================================
// Commands
// ==============================================================================

/// Activate Payload — SOL (NetFn=App 0x06, Cmd=0x48).
///
/// Activates a SOL payload instance on the BMC. This tells the BMC to start
/// accepting SOL payload data, but does not itself establish the bidirectional
/// terminal stream (that requires transport-level SOL payload handling).
///
/// # Errors
///
/// Returns an error if the transport fails, the BMC returns a non-success
/// completion code, or the response is too short to parse.
pub async fn activate_sol(
    transport: &mut impl IpmiTransport,
    instance: u8,
    encrypt: bool,
    authenticate: bool,
) -> Result<SolActivation> {
    // Request data: payload type (0x01 = SOL), payload instance,
    // encryption/authentication flags, and three reserved bytes.
    let mut flags: u8 = 0x00;
    if encrypt {
        flags |= 0x40; // bit 6 = encryption
    }
    if authenticate {
        flags |= 0x80; // bit 7 = authentication
    }

    let data = vec![
        0x01,     // payload type = SOL
        instance, // payload instance (typically 1)
        flags,    // encryption + authentication flags
        0x00,     // reserved
        0x00,     // reserved
        0x00,     // reserved
    ];

    let req = IpmiRequest::with_data(NetFn::App, 0x48, data);
    let resp = transport.send_recv(&req).await?;
    resp.check_completion()?;

    // Response: aux_data(4) + inbound_payload_size(2) + outbound_payload_size(2)
    //         + port(2) + vlan(2) = 12 bytes minimum.
    if resp.data.len() < 12 {
        return Err(IpmitoolError::InvalidResponse(format!(
            "Activate SOL response too short: expected >= 12 bytes, got {}",
            resp.data.len()
        )));
    }

    let mut aux_data = [0u8; 4];
    aux_data.copy_from_slice(&resp.data[0..4]);

    Ok(SolActivation {
        aux_data,
        inbound_payload_size: u16::from_le_bytes([resp.data[4], resp.data[5]]),
        outbound_payload_size: u16::from_le_bytes([resp.data[6], resp.data[7]]),
        port: u16::from_le_bytes([resp.data[8], resp.data[9]]),
        vlan: u16::from_le_bytes([resp.data[10], resp.data[11]]),
    })
}

/// Deactivate Payload — SOL (NetFn=App 0x06, Cmd=0x49).
///
/// Deactivates the given SOL payload instance, releasing BMC resources.
///
/// # Errors
///
/// Returns an error if the transport fails or the BMC returns a non-success
/// completion code.
pub async fn deactivate_sol(
    transport: &mut impl IpmiTransport,
    instance: u8,
) -> Result<()> {
    let data = vec![
        0x01,     // payload type = SOL
        instance, // payload instance
        0x00,     // reserved
        0x00,     // reserved
        0x00,     // reserved
    ];

    let req = IpmiRequest::with_data(NetFn::App, 0x49, data);
    let resp = transport.send_recv(&req).await?;
    resp.check_completion()
}

/// Get SOL Configuration Parameters (NetFn=Transport 0x0C, Cmd=0x22).
///
/// Reads parameters 0x01 through 0x06 to build a complete [`SolConfig`].
/// Each parameter requires a separate IPMI request.
///
/// # Errors
///
/// Returns an error if any transport call fails, the BMC returns a non-success
/// completion code, or any response is too short.
pub async fn get_sol_config(
    transport: &mut impl IpmiTransport,
    channel: u8,
) -> Result<SolConfig> {
    // Helper to issue a single Get SOL Config Parameter request.
    // Request: [channel (bit 7 = don't get revision only), param_selector,
    //           set_selector, block_selector]
    // Response: [parameter_revision, parameter_data...]

    // Parameter 0x01: SOL Enable
    let p01 = get_sol_param(transport, channel, 0x01).await?;
    if p01.is_empty() {
        return Err(IpmitoolError::InvalidResponse(
            "SOL Enable parameter (0x01) returned no data".to_string(),
        ));
    }
    let enabled = (p01[0] & 0x01) != 0;

    // Parameter 0x02: SOL Authentication
    let p02 = get_sol_param(transport, channel, 0x02).await?;
    if p02.is_empty() {
        return Err(IpmitoolError::InvalidResponse(
            "SOL Authentication parameter (0x02) returned no data".to_string(),
        ));
    }
    let privilege_level = p02[0] & 0x0F;
    let force_encryption = (p02[0] & 0x80) != 0;
    let force_authentication = (p02[0] & 0x40) != 0;

    // Parameter 0x03: Character Accumulate Interval & Send Threshold
    let p03 = get_sol_param(transport, channel, 0x03).await?;
    if p03.len() < 2 {
        return Err(IpmitoolError::InvalidResponse(format!(
            "SOL Char Accumulate parameter (0x03) too short: expected >= 2, got {}",
            p03.len()
        )));
    }
    // Interval is in 5ms increments; we store the raw value multiplied out.
    let char_accumulate_interval_ms = u16::from(p03[0]) * 5;
    let char_send_threshold = p03[1];

    // Parameter 0x04: SOL Retry
    let p04 = get_sol_param(transport, channel, 0x04).await?;
    if p04.len() < 2 {
        return Err(IpmitoolError::InvalidResponse(format!(
            "SOL Retry parameter (0x04) too short: expected >= 2, got {}",
            p04.len()
        )));
    }
    let retry_count = p04[0] & 0x07;
    // Interval is in 10ms increments.
    let retry_interval_ms = u16::from(p04[1]) * 10;

    // Parameter 0x05: Non-volatile Bit Rate
    let p05 = get_sol_param(transport, channel, 0x05).await?;
    if p05.is_empty() {
        return Err(IpmitoolError::InvalidResponse(
            "SOL Non-volatile Bit Rate parameter (0x05) returned no data".to_string(),
        ));
    }
    let non_volatile_bit_rate = SolBitRate::from_byte(p05[0] & 0x0F);

    // Parameter 0x06: Volatile Bit Rate
    let p06 = get_sol_param(transport, channel, 0x06).await?;
    if p06.is_empty() {
        return Err(IpmitoolError::InvalidResponse(
            "SOL Volatile Bit Rate parameter (0x06) returned no data".to_string(),
        ));
    }
    let volatile_bit_rate = SolBitRate::from_byte(p06[0] & 0x0F);

    Ok(SolConfig {
        enabled,
        privilege_level,
        force_encryption,
        force_authentication,
        char_accumulate_interval_ms,
        char_send_threshold,
        retry_count,
        retry_interval_ms,
        non_volatile_bit_rate,
        volatile_bit_rate,
    })
}

/// Set SOL Configuration Parameters — Enable (NetFn=Transport 0x0C, Cmd=0x21).
///
/// Sets parameter 0x01 to enable or disable SOL on the given channel.
///
/// # Errors
///
/// Returns an error if the transport fails or the BMC returns a non-success
/// completion code.
pub async fn set_sol_enable(
    transport: &mut impl IpmiTransport,
    channel: u8,
    enable: bool,
) -> Result<()> {
    let data = vec![
        channel,                          // channel number
        0x01,                             // parameter selector = SOL Enable
        if enable { 0x01 } else { 0x00 }, // enable/disable
    ];

    let req = IpmiRequest::with_data(NetFn::Transport, 0x21, data);
    let resp = transport.send_recv(&req).await?;
    resp.check_completion()
}

/// Set SOL Configuration Parameters — Bit Rate (NetFn=Transport 0x0C, Cmd=0x21).
///
/// Sets parameter 0x05 (non-volatile) or 0x06 (volatile) to the given rate.
///
/// # Errors
///
/// Returns an error if the transport fails or the BMC returns a non-success
/// completion code.
pub async fn set_sol_bit_rate(
    transport: &mut impl IpmiTransport,
    channel: u8,
    volatile: bool,
    rate: SolBitRate,
) -> Result<()> {
    let param_selector = if volatile { 0x06 } else { 0x05 };
    let data = vec![
        channel,        // channel number
        param_selector, // parameter selector
        rate.as_byte(), // bit rate value
    ];

    let req = IpmiRequest::with_data(NetFn::Transport, 0x21, data);
    let resp = transport.send_recv(&req).await?;
    resp.check_completion()
}

// ==============================================================================
// Internal Helpers
// ==============================================================================

/// Issue a single Get SOL Configuration Parameter request and return the
/// parameter data bytes (excluding the parameter revision byte).
async fn get_sol_param(
    transport: &mut impl IpmiTransport,
    channel: u8,
    param_selector: u8,
) -> Result<Vec<u8>> {
    let data = vec![
        channel,        // channel (bit 7 clear = return parameter data)
        param_selector, // parameter selector
        0x00,           // set selector
        0x00,           // block selector
    ];

    let req = IpmiRequest::with_data(NetFn::Transport, 0x22, data);
    let resp = transport.send_recv(&req).await?;
    resp.check_completion()?;

    // Response byte 0 is parameter revision; the rest is the parameter data.
    if resp.data.is_empty() {
        return Err(IpmitoolError::InvalidResponse(format!(
            "Get SOL Config param 0x{param_selector:02X} response empty"
        )));
    }

    Ok(resp.data[1..].to_vec())
}

// ==============================================================================
// Tests
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::mock::QueueMockTransport;

    #[tokio::test]
    async fn activate_sol_parses_response() {
        let mut transport = QueueMockTransport::new();

        // Activate SOL response: completion_code + 12 bytes of payload data.
        // aux_data = [0x01, 0x02, 0x03, 0x04]
        // inbound_payload_size = 0x00FC (252) little-endian
        // outbound_payload_size = 0x00FC (252) little-endian
        // port = 0x026F (623) little-endian
        // vlan = 0xFFFF (no VLAN) little-endian
        transport.enqueue(
            0x06,
            0x48,
            vec![
                0x00, // completion code
                0x01, 0x02, 0x03, 0x04, // aux_data
                0xFC, 0x00, // inbound_payload_size = 252
                0xFC, 0x00, // outbound_payload_size = 252
                0x6F, 0x02, // port = 623
                0xFF, 0xFF, // vlan = 0xFFFF
            ],
        );

        let activation = activate_sol(&mut transport, 1, true, true)
            .await
            .expect("should parse activation response");

        assert_eq!(activation.aux_data, [0x01, 0x02, 0x03, 0x04]);
        assert_eq!(activation.inbound_payload_size, 252);
        assert_eq!(activation.outbound_payload_size, 252);
        assert_eq!(activation.port, 623);
        assert_eq!(activation.vlan, 0xFFFF);
    }

    #[tokio::test]
    async fn activate_sol_short_response_errors() {
        let mut transport = QueueMockTransport::new();

        // Response with only 8 data bytes (need 12).
        transport.enqueue(
            0x06,
            0x48,
            vec![0x00, 0x01, 0x02, 0x03, 0x04, 0xFC, 0x00, 0xFC, 0x00],
        );

        let result = activate_sol(&mut transport, 1, true, true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn deactivate_sol_sends_correct_request() {
        let mut transport = QueueMockTransport::new();

        // Success response (just completion code).
        transport.enqueue(0x06, 0x49, vec![0x00]);

        deactivate_sol(&mut transport, 1)
            .await
            .expect("deactivate should succeed");
    }

    #[tokio::test]
    async fn get_sol_config_reads_all_parameters() {
        let mut transport = QueueMockTransport::new();

        // Each Get SOL Config Parameter response has: completion_code,
        // parameter_revision, then parameter data.

        // Param 0x01 (SOL Enable): enabled = true
        transport.enqueue(0x0C, 0x22, vec![0x00, 0x11, 0x01]);

        // Param 0x02 (SOL Authentication): force_encrypt=1, force_auth=1, priv=admin(4)
        transport.enqueue(0x0C, 0x22, vec![0x00, 0x11, 0xC4]);

        // Param 0x03 (Char Accumulate): interval=10 (50ms), threshold=96
        transport.enqueue(0x0C, 0x22, vec![0x00, 0x11, 0x0A, 0x60]);

        // Param 0x04 (SOL Retry): count=3, interval=25 (250ms)
        transport.enqueue(0x0C, 0x22, vec![0x00, 0x11, 0x03, 0x19]);

        // Param 0x05 (Non-volatile Bit Rate): 115200 = 0x0A
        transport.enqueue(0x0C, 0x22, vec![0x00, 0x11, 0x0A]);

        // Param 0x06 (Volatile Bit Rate): 19200 = 0x07
        transport.enqueue(0x0C, 0x22, vec![0x00, 0x11, 0x07]);

        let config = get_sol_config(&mut transport, 1)
            .await
            .expect("should parse all SOL config parameters");

        assert!(config.enabled);
        assert_eq!(config.privilege_level, 0x04);
        assert!(config.force_encryption);
        assert!(config.force_authentication);
        assert_eq!(config.char_accumulate_interval_ms, 50);
        assert_eq!(config.char_send_threshold, 0x60);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_interval_ms, 250);
        assert_eq!(config.non_volatile_bit_rate, SolBitRate::Rate115200);
        assert_eq!(config.volatile_bit_rate, SolBitRate::Rate19200);
    }

    #[tokio::test]
    async fn set_sol_enable_sends_correct_request() {
        let mut transport = QueueMockTransport::new();
        transport.enqueue(0x0C, 0x21, vec![0x00]);

        set_sol_enable(&mut transport, 1, true)
            .await
            .expect("set SOL enable should succeed");
    }

    #[tokio::test]
    async fn set_sol_bit_rate_volatile() {
        let mut transport = QueueMockTransport::new();
        transport.enqueue(0x0C, 0x21, vec![0x00]);

        set_sol_bit_rate(&mut transport, 1, true, SolBitRate::Rate115200)
            .await
            .expect("set volatile bit rate should succeed");
    }

    #[tokio::test]
    async fn set_sol_bit_rate_non_volatile() {
        let mut transport = QueueMockTransport::new();
        transport.enqueue(0x0C, 0x21, vec![0x00]);

        set_sol_bit_rate(&mut transport, 1, false, SolBitRate::Rate9600)
            .await
            .expect("set non-volatile bit rate should succeed");
    }

    #[tokio::test]
    async fn bit_rate_roundtrip() {
        let rates = [
            SolBitRate::UseDefault,
            SolBitRate::Rate9600,
            SolBitRate::Rate19200,
            SolBitRate::Rate38400,
            SolBitRate::Rate57600,
            SolBitRate::Rate115200,
        ];
        for rate in rates {
            let byte = rate.as_byte();
            let back = SolBitRate::from_byte(byte);
            assert_eq!(rate, back, "roundtrip failed for {rate:?}");
        }
    }

    #[tokio::test]
    async fn bit_rate_unknown_preserved() {
        let rate = SolBitRate::Unknown(0xFE);
        let byte = rate.as_byte();
        assert_eq!(byte, 0xFE);
        // from_byte(0xFE) should also yield Unknown since it's not a known rate.
        let back = SolBitRate::from_byte(0xFE);
        assert_eq!(back, SolBitRate::Unknown(0xFE));
    }
}
