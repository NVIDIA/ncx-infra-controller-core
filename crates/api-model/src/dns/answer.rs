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

use carbide_uuid::domain::DomainId;
use carbide_uuid::vpc::VpcId;
use dns_record::SoaRecord;

use super::resource_record::ResourceRecord;
use super::zone::Fqdn;

/// Which zonees does this answer belong to?
/// `Site` is a zone we own and can answer queries for.
/// `Delegated` is a zone we don't own, but we can answer queries for because we're delegated to by the zone's owner.
///
/// Authoritative outcomes (Records, NoData, NxDomain) all include an `AuthorityRef`. `NotAuthoritative`, `Refused`, and `NotImplemented` do not.
//
// Answer | Outcome | Authoritative (AA)
// ----------------------------------------------------------------------------
// Records (even empty) | Records | true
// NoData  | NoData  | true
// NxDomain | NoSuchName | true
// NotAuthoritative | NotAuthoritative | false
// Refused | Refused | false
// NotImplemented | NotImplemented | false
// ----------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum AuthorityRef {
    Site {
        domain: DomainId,
        zone: Fqdn,
    },
    Delegated {
        domain: DomainId,
        vpc: VpcId,
        zone: Fqdn,
    },
}

impl AuthorityRef {
    pub fn zone(&self) -> &Fqdn {
        match self {
            Self::Site { zone, .. } | Self::Delegated { zone, .. } => zone,
        }
    }
}

/// Query outcome before wire mapping. Empty `Records` is still `Records`, not NXDOMAIN.
#[derive(Clone, Debug)]
pub enum Answer {
    Records {
        authority: AuthorityRef,
        records: Vec<ResourceRecord>,
    },
    NoData {
        authority: AuthorityRef,
        soa: SoaRecord,
    },
    NxDomain {
        authority: AuthorityRef,
        soa: SoaRecord,
    },
    NotAuthoritative,
    Refused,
    NotImplemented,
}

/// gRPC lookup fields that `carbide-dns` maps to RCODE and AA. Not a DNS RCODE number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupOutcome {
    Records,
    NoData,
    NoSuchName,
    NotAuthoritative,
    Refused,
    NotImplemented,
}

// The two flags required to build a DNS response
// What happened(outcome) and whether we are the zone owner (authoritative)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LookupWire {
    pub outcome: LookupOutcome,
    pub authoritative: bool,
}

impl Answer {
    pub fn lookup_wire(&self) -> LookupWire {
        match self {
            Self::Records { .. } => LookupWire {
                outcome: LookupOutcome::Records,
                authoritative: true,
            },
            Self::NoData { .. } => LookupWire {
                outcome: LookupOutcome::NoData,
                authoritative: true,
            },
            Self::NxDomain { .. } => LookupWire {
                outcome: LookupOutcome::NoSuchName,
                authoritative: true,
            },
            Self::NotAuthoritative => LookupWire {
                outcome: LookupOutcome::NotAuthoritative,
                authoritative: false,
            },
            Self::Refused => LookupWire {
                outcome: LookupOutcome::Refused,
                authoritative: false,
            },
            Self::NotImplemented => LookupWire {
                outcome: LookupOutcome::NotImplemented,
                authoritative: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use carbide_test_support::value_scenarios;
    use carbide_uuid::domain::DomainId;
    use dns_record::SoaRecord;

    use super::*;
    use crate::dns::zone::Fqdn;

    fn site() -> AuthorityRef {
        AuthorityRef::Site {
            domain: DomainId::from(uuid::Uuid::new_v4()),
            zone: Fqdn::parse("mysite.example.com").expect("fixture zone"),
        }
    }

    fn soa() -> SoaRecord {
        SoaRecord::new("mysite.example.com")
    }

    #[test]
    fn lookup_wire_matches_rcode_aa_matrix() {
        let authority = site();
        value_scenarios!(
            run = |answer: Answer| answer.lookup_wire();
            "in-zone RRset" {
                Answer::Records {
                    authority: authority.clone(),
                    records: vec![],
                } => LookupWire {
                    outcome: LookupOutcome::Records,
                    authoritative: true,
                },
            }
            "name exists, no type" {
                Answer::NoData {
                    authority: authority.clone(),
                    soa: soa(),
                } => LookupWire {
                    outcome: LookupOutcome::NoData,
                    authoritative: true,
                },
            }
            "in-zone, name missing" {
                Answer::NxDomain {
                    authority: authority.clone(),
                    soa: soa(),
                } => LookupWire {
                    outcome: LookupOutcome::NoSuchName,
                    authoritative: true,
                },
            }
            "no held zone contains qname" {
                Answer::NotAuthoritative => LookupWire {
                    outcome: LookupOutcome::NotAuthoritative,
                    authoritative: false,
                },
            }
            "we hold the zone, will not perform the operation" {
                Answer::Refused => LookupWire {
                    outcome: LookupOutcome::Refused,
                    authoritative: false,
                },
            }
            "never implemented" {
                Answer::NotImplemented => LookupWire {
                    outcome: LookupOutcome::NotImplemented,
                    authoritative: false,
                },
            }
        );
    }
}
