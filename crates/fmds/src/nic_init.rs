/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::time::Duration;

use eyre::eyre;
use futures_util::TryStreamExt;
use ipnetwork::IpNetwork;
use netlink_packet_route::address::AddressAttribute;
use rtnetlink::Handle;

const LINK_LOOKUP_RETRIES: u32 = 15;
const LINK_LOOKUP_BACKOFF: Duration = Duration::from_secs(2);

/// Assign `cidr` to the interface named `name` and bring it up. Idempotent:
/// if the address is already present, the add is skipped. Retries while the
/// interface is not yet attached (Multus may attach the NIC slightly after
/// the pod starts).
pub async fn assign_address(name: &str, cidr: IpNetwork) -> eyre::Result<()> {
    let (connection, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(connection);

    let index = wait_for_link(&handle, name).await?;

    if address_already_present(&handle, index, cidr).await? {
        tracing::info!(interface = name, %cidr, "address already assigned; skipping add");
    } else {
        handle
            .address()
            .add(index, cidr.ip(), cidr.prefix())
            .execute()
            .await?;
        tracing::info!(interface = name, %cidr, "assigned address");
    }

    handle.link().set(index).up().execute().await?;
    Ok(())
}

async fn wait_for_link(handle: &Handle, name: &str) -> eyre::Result<u32> {
    for attempt in 1..=LINK_LOOKUP_RETRIES {
        let mut links = handle.link().get().match_name(name.to_string()).execute();
        match links.try_next().await {
            Ok(Some(link)) => return Ok(link.header.index),
            Ok(None) | Err(rtnetlink::Error::NetlinkError(_)) => {
                tracing::debug!(
                    interface = name,
                    attempt,
                    "interface not yet present, retrying"
                );
            }
            Err(e) => return Err(e.into()),
        }
        tokio::time::sleep(LINK_LOOKUP_BACKOFF).await;
    }
    Err(eyre!(
        "interface {name} not found after {LINK_LOOKUP_RETRIES} attempts"
    ))
}

async fn address_already_present(
    handle: &Handle,
    index: u32,
    cidr: IpNetwork,
) -> eyre::Result<bool> {
    let mut addrs = handle
        .address()
        .get()
        .set_link_index_filter(index)
        .execute();
    while let Some(msg) = addrs.try_next().await? {
        let prefix_len = msg.header.prefix_len;
        for attr in &msg.attributes {
            if let AddressAttribute::Address(ip) = attr
                && *ip == cidr.ip()
                && prefix_len == cidr.prefix()
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
