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
use ::rpc::protos;
use db::db_read::DbReader;
use db::dns::resource_record;
use dns_record::{DnsResourceRecordReply, DnsResourceRecordType, SoaRecord};
use model::dns::{Answer, AuthorityRef, Fqdn, LookupOutcome, ResourceRecord};
use tonic::{Request, Response, Status};

use crate::CarbideError;
use crate::api::{Api, log_request_data};

#[derive(Clone, Debug)]
struct DnsResourceRecordLookupResponse {
    record: Vec<DnsResourceRecordReply>,
    outcome: LookupOutcome,
    authoritative: bool,
    authority_soa: Option<DnsResourceRecordReply>,
}

impl DnsResourceRecordLookupResponse {
    fn new(
        record: Vec<DnsResourceRecordReply>,
        outcome: LookupOutcome,
        authoritative: bool,
        authority_soa: Option<DnsResourceRecordReply>,
    ) -> Self {
        Self {
            record,
            outcome,
            authoritative,
            authority_soa,
        }
    }

    fn from_answer(answer: Answer) -> Self {
        let wire = answer.lookup_wire();
        match answer {
            Answer::Records { records, .. } => Self::new(
                records.into_iter().map(Into::into).collect(),
                wire.outcome,
                wire.authoritative,
                None,
            ),
            Answer::NoData { authority, soa } => Self::new(
                vec![],
                wire.outcome,
                wire.authoritative,
                Some(authority_soa_reply(authority.zone(), &soa)),
            ),
            Answer::NxDomain { authority, soa } => Self::new(
                vec![],
                wire.outcome,
                wire.authoritative,
                Some(authority_soa_reply(authority.zone(), &soa)),
            ),
            Answer::NotAuthoritative | Answer::Refused | Answer::NotImplemented => {
                Self::new(vec![], wire.outcome, wire.authoritative, None)
            }
        }
    }
}

fn authority_soa_reply(zone: &Fqdn, soa: &SoaRecord) -> DnsResourceRecordReply {
    DnsResourceRecordReply {
        qtype: DnsResourceRecordType::SOA.to_string(),
        qname: zone.as_str().to_string(),
        ttl: soa.ttl.0 as u32,
        content: soa.to_string(),
        domain_id: None,
        scope_mask: None,
        auth: None,
    }
}

/// Publish a zone SOA as a lookup `ResourceRecord` at the zone apex.
fn soa_as_model(authority: &AuthorityRef, soa: &SoaRecord) -> ResourceRecord {
    ResourceRecord {
        q_type: DnsResourceRecordType::SOA.to_string(),
        q_name: authority.zone().as_str().to_string(),
        ttl: soa.ttl.0 as u32,
        content: soa.to_string(),
        domain_id: None,
    }
}

/// A `HeldAuthority` is a DNS zone that we actually own and can answer queries for.
///
/// It includes the zones `AuthorityRef`, the zone's SOA record, and a boolean for whether we're asking exactly at that zone's name.
/// If we're asking exactly at that zone's name, we can return the zone's SOA record directly,
/// otherwise we need to look up the records in the zone.
struct HeldAuthority {
    authority: AuthorityRef,
    soa: Option<SoaRecord>,
    is_apex: bool,
}

/// Find the longest-suffix `HeldAuthority` that contains `qname`.
///
/// Load every `domain` from `domains` table that is not deleted. We keep the `domains` that `qname` is within — same name, or a hostname under it.
/// If more than one matches, we take the longest name, because that’s the most specific zone. If none match, we don’t own this name.
/// If we find a match, we return the `HeldAuthority` - the zone's `AuthorityRef` and `SoaRecord`, and a boolean for whether we're asking exactly at that zone's name.
/// If we don't find a match, we return `None`.
async fn find_site_authority(
    db: impl DbReader<'_>,
    qname: &Fqdn,
) -> Result<Option<HeldAuthority>, Status> {
    let domains =
        db::dns::domain::find_by(db, db::ObjectColumnFilter::<db::dns::domain::IdColumn>::All)
            .await
            .map_err(CarbideError::from)?;

    let closest = domains
        .into_iter()
        .filter_map(|domain| {
            let zone = Fqdn::parse(&domain.name).ok()?;
            qname.is_within(&zone).then_some((domain, zone))
        })
        .max_by_key(|(_, zone)| zone.as_str().len());

    let Some((domain, zone)) = closest else {
        return Ok(None);
    };
    Ok(Some(HeldAuthority {
        is_apex: qname == &zone,
        authority: AuthorityRef::Site {
            domain: domain.id,
            zone,
        },
        soa: domain.soa.map(|soa| soa.0),
    }))
}

impl From<DnsResourceRecordLookupResponse> for protos::dns::DnsResourceRecordLookupResponse {
    fn from(value: DnsResourceRecordLookupResponse) -> Self {
        Self {
            records: value.record.into_iter().map(Into::into).collect(),
            outcome: protos::dns::DnsLookupOutcome::from(value.outcome) as i32,
            authority_soa: value.authority_soa.map(Into::into),
            authoritative: value.authoritative,
        }
    }
}

/// Returns all published record types for a qname. The authority layer selects the requested type.
async fn lookup_records_by_qname(
    txn: impl DbReader<'_>,
    query_name: &str,
) -> Result<Vec<ResourceRecord>, tonic::Status> {
    tracing::debug!(query_name, "Looking up DNS records",);

    // dns_records view expects trailing dots (FQDN format)
    let qname_with_dot = if !query_name.ends_with('.') {
        format!("{}.", query_name)
    } else {
        query_name.to_string()
    };

    let result = resource_record::find_record(txn, &qname_with_dot)
        .await
        .map_err(CarbideError::from)?
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>();

    Ok(result)
}

/// Resolve a reverse-DNS (PTR) query. The qname is an address in `in-addr.arpa` /
/// `ip6.arpa` form, so we parse it back to an `IpAddr` and look the holding
/// interface up by address (rather than matching a per-row arpa string in a view).
/// An unparseable name, or one no interface holds, yields no records.
async fn lookup_ptr_record(
    txn: impl DbReader<'_>,
    query_name: &str,
) -> Result<Vec<ResourceRecord>, tonic::Status> {
    tracing::debug!(qname = %query_name, "looking up PTR record");

    let qname_with_dot = if !query_name.ends_with('.') {
        format!("{}.", query_name)
    } else {
        query_name.to_string()
    };

    let Some(address) = db::dns::arpa_qname_to_ip(&qname_with_dot) else {
        return Ok(vec![]);
    };

    let result = resource_record::find_ptr_record(txn, address)
        .await
        .map_err(CarbideError::from)?
        .into_iter()
        .map(|record| ResourceRecord {
            q_type: DnsResourceRecordType::PTR.to_string(),
            q_name: qname_with_dot.clone(),
            ttl: record.ttl as u32,
            content: record.ptr_content,
            domain_id: Some(record.domain_id.to_string()),
        })
        .collect::<Vec<_>>();

    Ok(result)
}

async fn lookup_answer(
    db: impl DbReader<'_> + Copy,
    qname: &str,
    qtype: DnsResourceRecordType,
) -> Result<Answer, Status> {
    let qname =
        Fqdn::parse(qname).map_err(|error| CarbideError::InvalidArgument(error.to_string()))?;
    let Some(held) = find_site_authority(db, &qname).await? else {
        return Ok(Answer::NotAuthoritative);
    };

    if held.is_apex && qtype == DnsResourceRecordType::SOA {
        let soa = held.soa.ok_or_else(|| {
            Status::internal(format!(
                "held DNS zone {} does not have an SOA record",
                held.authority.zone().as_str()
            ))
        })?;
        let record = soa_as_model(&held.authority, &soa);
        return Ok(Answer::Records {
            authority: held.authority,
            records: vec![record],
        });
    }

    let all_records = if qtype == DnsResourceRecordType::PTR {
        lookup_ptr_record(db, qname.as_str()).await?
    } else {
        lookup_records_by_qname(db, qname.as_str()).await?
    };

    Ok(Answer::Records {
        authority: held.authority,
        records: all_records,
    })
}

pub(crate) async fn get_all_domains(
    api: &Api,
    _request: Request<protos::dns::GetAllDomainsRequest>,
) -> Result<Response<protos::dns::GetAllDomainsResponse>, Status> {
    log_request_data(&_request);

    let domains = db::dns::domain::find_by(
        &api.database_connection,
        db::ObjectColumnFilter::<db::dns::domain::IdColumn>::All,
    )
    .await?;

    tracing::debug!(domain_count = domains.len(), "Found domains");
    for domain in &domains {
        tracing::debug!(
            domain_id = %domain.id,
            domain_name = %domain.name,
            "Domain"
        );
    }

    let result: Vec<protos::dns::DomainInfo> = domains
        .into_iter()
        .map(model::dns::DomainInfo::from)
        .map(protos::dns::DomainInfo::from)
        .collect();

    let response = protos::dns::GetAllDomainsResponse { result };

    tracing::debug!(
        domain_info_count = response.result.len(),
        "Formatted DomainInfo response"
    );
    Ok(Response::new(response))
}

pub(crate) async fn get_all_domain_metadata(
    api: &Api,
    request: Request<protos::dns::DomainMetadataRequest>,
) -> Result<Response<protos::dns::DomainMetadataResponse>, Status> {
    log_request_data(&request);

    let metadata_request = request.into_inner();

    let domain_name = db::dns::normalize_domain(&metadata_request.domain);

    // Reverse zones may be stored with or without the trailing root dot, so
    // resolve their normalized identity. Forward domains retain the existing
    // exact lookup after the request normalization above.
    let domains = db::dns::domain::find_by_name(&api.database_connection, &domain_name).await?;

    let domain = domains.first().ok_or_else(|| CarbideError::NotFoundError {
        kind: "domain",
        id: metadata_request.domain.clone(),
    })?;

    let proto_metadata = domain
        .metadata
        .as_ref()
        .map(|m| protos::dns::Metadata::from(m.clone()));

    Ok(Response::new(protos::dns::DomainMetadataResponse {
        result: proto_metadata,
    }))
}
pub(crate) async fn lookup_record(
    api: &Api,
    request: Request<protos::dns::DnsResourceRecordLookupRequest>,
) -> Result<Response<protos::dns::DnsResourceRecordLookupResponse>, Status> {
    log_request_data(&request);

    let lookup_request = request.into_inner();

    // Log the full incoming request for debugging
    tracing::debug!(
        qtype = %lookup_request.qtype,
        qname = %lookup_request.qname,
        zone_id = %lookup_request.zone_id,
        "Processing DNS lookup request"
    );

    let rrtype = DnsResourceRecordType::try_from(lookup_request.qtype)
        .map_err(|e| CarbideError::InvalidArgument(format!("invalid qtype supplied: {}", e)))?;

    let qname = lookup_request.qname;

    if qname.is_empty() {
        return Err(CarbideError::InvalidArgument("qname cannot be empty".to_string()).into());
    }

    let answer = lookup_answer(&api.database_connection, &qname, rrtype).await?;
    let resp = DnsResourceRecordLookupResponse::from_answer(answer);
    Ok(Response::new(resp.into()))
}

#[cfg(test)]
mod tests {
    use carbide_test_support::value_scenarios;
    use carbide_uuid::domain::DomainId;

    use super::*;

    #[test]
    fn lookup_response_mapping_copies_answer_metadata() {
        let authority = site_authority();
        value_scenarios!(
            run = |answer: Answer| {
                let proto = protos::dns::DnsResourceRecordLookupResponse::from(
                    DnsResourceRecordLookupResponse::from_answer(answer),
                );
                (
                    proto.outcome,
                    proto.authoritative,
                    proto.authority_soa.is_some(),
                    proto.records.len(),
                )
            };
            "in-zone RRset" {
                Answer::Records {
                    authority: authority.clone(),
                    records: vec![model_a_record()],
                } => (
                    protos::dns::DnsLookupOutcome::Records as i32,
                    true,
                    false,
                    1,
                ),
            }
            "empty Records is Records, not NXDOMAIN" {
                Answer::Records {
                    authority: authority.clone(),
                    records: vec![],
                } => (
                    protos::dns::DnsLookupOutcome::Records as i32,
                    true,
                    false,
                    0,
                ),
            }
            "NODATA carries authority SOA" {
                Answer::NoData {
                    authority: authority.clone(),
                    soa: dns_record::SoaRecord::new("mysite.example.com"),
                } => (
                    protos::dns::DnsLookupOutcome::NoData as i32,
                    true,
                    true,
                    0,
                ),
            }
            "not authoritative" {
                Answer::NotAuthoritative => (
                    protos::dns::DnsLookupOutcome::NotAuthoritative as i32,
                    false,
                    false,
                    0,
                ),
            }
        );
    }

    fn site_authority() -> AuthorityRef {
        AuthorityRef::Site {
            domain: DomainId::from(uuid::Uuid::nil()),
            zone: Fqdn::parse("mysite.example.com").expect("fixture zone"),
        }
    }

    fn model_a_record() -> ResourceRecord {
        ResourceRecord {
            q_type: DnsResourceRecordType::A.to_string(),
            q_name: "gpu.mysite.example.com.".to_string(),
            ttl: 300,
            content: "192.0.2.10".to_string(),
            domain_id: None,
        }
    }
}
