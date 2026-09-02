/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::net::IpAddr;

use dns_record::DnsResourceRecordType;
use thiserror::Error;

use super::zone::{Fqdn, NameError};

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum QuestionError {
    #[error("{0}")]
    Name(#[from] NameError),
    #[error("unknown qtype {0}")]
    UnknownQtype(String),
    #[error("PTR qname is not a well-formed reverse name")]
    NotArpa,
}

/// Parsed lookup. Apex vs in-zone SOA/NS is decided by the authority, not parse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Question {
    Forward {
        name: Fqdn,
        qtype: DnsResourceRecordType,
    },
    Reverse {
        name: Fqdn,
        address: IpAddr,
    },
    ApexSoa {
        zone: Fqdn,
    },
    ApexNs {
        zone: Fqdn,
    },
}

impl Question {
    pub fn parse(qname: &str, qtype: &str) -> Result<Self, QuestionError> {
        let name = Fqdn::parse(qname)?;
        let parsed_type = DnsResourceRecordType::try_from(qtype)
            .map_err(|_| QuestionError::UnknownQtype(qtype.to_string()))?;
        match parsed_type {
            DnsResourceRecordType::SOA => Ok(Self::ApexSoa { zone: name }),
            DnsResourceRecordType::NS => Ok(Self::ApexNs { zone: name }),
            DnsResourceRecordType::PTR => {
                let address = arpa_qname_to_ip(name.as_str()).ok_or(QuestionError::NotArpa)?;
                Ok(Self::Reverse { name, address })
            }
            qtype => Ok(Self::Forward { name, qtype }),
        }
    }
}

/// Inverse of the `in-addr.arpa` / `ip6.arpa` presentation form. `None` if the name is not arpa.
fn arpa_qname_to_ip(qname: &str) -> Option<IpAddr> {
    use std::net::{Ipv4Addr, Ipv6Addr};

    let name = qname.trim_end_matches('.').to_ascii_lowercase();

    if let Some(reversed) = name.strip_suffix(".in-addr.arpa") {
        let octets: Vec<&str> = reversed.split('.').collect();
        if octets.len() != 4 {
            return None;
        }
        let mut addr = [0u8; 4];
        for (byte, octet) in addr.iter_mut().zip(octets.iter().rev()) {
            *byte = octet.parse().ok()?;
        }
        Some(IpAddr::V4(Ipv4Addr::from(addr)))
    } else if let Some(reversed) = name.strip_suffix(".ip6.arpa") {
        let nibbles: Vec<&str> = reversed.split('.').collect();
        if nibbles.len() != 32 {
            return None;
        }
        let mut addr = [0u8; 16];
        for (i, nibble) in nibbles.iter().rev().enumerate() {
            if nibble.len() != 1 {
                return None;
            }
            let value = u8::from_str_radix(nibble, 16).ok()?;
            if i % 2 == 0 {
                addr[i / 2] = value << 4;
            } else {
                addr[i / 2] |= value;
            }
        }
        Some(IpAddr::V6(Ipv6Addr::from(addr)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use carbide_test_support::Outcome::{FailsWith, Yields};
    use carbide_test_support::scenarios;
    use dns_record::DnsResourceRecordType;

    use super::*;
    use crate::dns::zone::Fqdn;

    #[test]
    fn parse_classifies_qtype_without_deciding_authority() {
        scenarios!(
            run = |(qname, qtype)| Question::parse(qname, qtype);
            "forward and apex query types" {
                ("gpu.mysite.example.com.", "A") => Yields(Question::Forward {
                    name: Fqdn::parse("gpu.mysite.example.com.").unwrap(),
                    qtype: DnsResourceRecordType::A,
                }),
                ("gpu.mysite.example.com.", "MX") => Yields(Question::Forward {
                    name: Fqdn::parse("gpu.mysite.example.com.").unwrap(),
                    qtype: DnsResourceRecordType::MX,
                }),
                ("mysite.example.com.", "SOA") => Yields(Question::ApexSoa {
                    zone: Fqdn::parse("mysite.example.com.").unwrap(),
                }),
                ("mysite.example.com.", "NS") => Yields(Question::ApexNs {
                    zone: Fqdn::parse("mysite.example.com.").unwrap(),
                }),
            }
            "reverse" {
                ("4.3.2.1.in-addr.arpa.", "PTR") => Yields(Question::Reverse {
                    name: Fqdn::parse("4.3.2.1.in-addr.arpa.").unwrap(),
                    address: IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
                }),
            }
            "invalid" {
                ("gpu.mysite.example.com.", "SPAM") => FailsWith(QuestionError::UnknownQtype(
                    "SPAM".to_string(),
                )),
                ("not-arpa.example.com.", "PTR") => FailsWith(QuestionError::NotArpa),
            }
        );
    }
}
