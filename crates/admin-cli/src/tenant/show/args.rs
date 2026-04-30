/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use clap::Parser;
use rpc::forge::FindTenantRequest;

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[clap(help = "Optional, tenant org ID to restrict the search")]
    pub tenant_org: Option<String>,
}

impl From<&Args> for Option<FindTenantRequest> {
    fn from(args: &Args) -> Self {
        args.tenant_org.as_ref().map(|id| FindTenantRequest {
            tenant_organization_id: id.clone(),
        })
    }
}
