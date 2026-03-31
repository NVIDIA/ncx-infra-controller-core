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

use ::rpc::admin_cli::{CarbideCliError, OutputFormat};
use prettytable::{Cell, Row, Table};

use super::args::Args;
use crate::rpc::ApiClient;

fn scope_display(scope: i32) -> &'static str {
    match rpc::forge::IpxeScriptTemplateScope::try_from(scope) {
        Ok(rpc::forge::IpxeScriptTemplateScope::Internal) => "internal",
        Ok(rpc::forge::IpxeScriptTemplateScope::Public) => "public",
        _ => "unknown",
    }
}

pub async fn handle_show(
    opts: Args,
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    if opts.name.as_deref().unwrap_or("").is_empty() {
        list_all(format, api_client).await
    } else {
        show_one(opts.name.as_deref().unwrap(), format, api_client).await
    }
}

async fn list_all(
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let result = api_client
        .0
        .list_ipxe_script_templates()
        .await?;

    if format == OutputFormat::Json {
        println!("{}", serde_json::to_string_pretty(&result.templates)?);
    } else if result.templates.is_empty() {
        println!("No iPXE templates found.");
    } else {
        let mut table = Table::new();
        table.set_titles(Row::new(vec![
            Cell::new("Name"),
            Cell::new("Description"),
            Cell::new("Scope"),
            Cell::new("Required Params"),
            Cell::new("Required Artifacts"),
        ]));

        for tmpl in &result.templates {
            table.add_row(Row::new(vec![
                Cell::new(&tmpl.name),
                Cell::new(&tmpl.description),
                Cell::new(scope_display(tmpl.scope)),
                Cell::new(&tmpl.required_params.join(", ")),
                Cell::new(&tmpl.required_artifacts.join(", ")),
            ]));
        }

        table.printstd();
    }

    Ok(())
}

async fn show_one(
    name: &str,
    format: OutputFormat,
    api_client: &ApiClient,
) -> Result<(), CarbideCliError> {
    let result = match api_client
        .0
        .get_ipxe_script_template(rpc::forge::GetIpxeScriptTemplateRequest {
            name: name.to_string(),
        })
        .await
    {
        Ok(tmpl) => tmpl,
        Err(status) if status.code() == tonic::Code::NotFound => {
            return Err(CarbideCliError::GenericError(format!(
                "iPXE template not found: {}",
                name
            )));
        }
        Err(err) => return Err(CarbideCliError::from(err)),
    };

    if format == OutputFormat::Json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Name:        {}", result.name);
        println!("Description: {}", result.description);
        println!("Scope:       {}", scope_display(result.scope));

        if !result.required_params.is_empty() {
            println!("Required params:    {}", result.required_params.join(", "));
        }
        if !result.reserved_params.is_empty() {
            println!("Reserved params:    {}", result.reserved_params.join(", "));
        }
        if !result.required_artifacts.is_empty() {
            println!(
                "Required artifacts: {}",
                result.required_artifacts.join(", ")
            );
        }

        println!("\nTemplate:\n---\n{}\n---", result.template);
    }

    Ok(())
}
