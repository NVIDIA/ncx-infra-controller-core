/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::power_shelf::PowerShelfId;
use clap::{ArgGroup, Parser};

use crate::machine::HealthReportTemplates;

#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("health_report_source").required(true).args(&["health_report", "template"])))]
pub struct Args {
    pub power_shelf_id: PowerShelfId,
    #[clap(long, help = "New health report as json")]
    pub health_report: Option<String>,
    #[clap(long, help = "Predefined Template name")]
    pub template: Option<HealthReportTemplates>,
    #[clap(long, help = "Message to be filled in template.")]
    pub message: Option<String>,
    #[clap(long, help = "Replace all other health reports with this source")]
    pub replace: bool,
    #[clap(long, help = "Print the template that is going to be send to carbide")]
    pub print_only: bool,
}
