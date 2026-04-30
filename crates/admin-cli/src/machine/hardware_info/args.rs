/*
 * SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use carbide_uuid::machine::MachineId;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum Args {
    #[clap(about = "Show the hardware info of the machine")]
    Show(ShowMachineHardwareInfo),
    #[clap(subcommand, about = "Update the hardware info of the machine")]
    Update(MachineHardwareInfo),
}

#[derive(Parser, Debug)]
pub struct ShowMachineHardwareInfo {
    #[clap(long, help = "Show the hardware info of this Machine ID")]
    pub machine: MachineId,
}

#[derive(Parser, Debug)]
pub enum MachineHardwareInfo {
    //Cpu(MachineTopologyCommandCpu),
    #[clap(about = "Update the GPUs of this machine")]
    Gpus(MachineHardwareInfoGpus),
    //Memory(MachineTopologyCommandMemory),
    //Storage(MachineTopologyCommandStorage),
    //Network(MachineTopologyCommandNetwork),
    //Infiniband(MachineTopologyCommandInfiniband),
    //Dpu(MachineTopologyCommandDpu),
}

#[derive(Parser, Debug)]
pub struct MachineHardwareInfoGpus {
    #[clap(long, help = "Machine ID of the server containing the GPUs")]
    pub machine: MachineId,
    #[clap(
        long,
        help = "JSON file containing GPU info. It should contain an array of JSON objects like this:
        {
            \"name\": \"string\",
            \"serial\": \"string\",
            \"driver_version\": \"string\",
            \"vbios_version\": \"string\",
            \"inforom_version\": \"string\",
            \"total_memory\": \"string\",
            \"frequency\": \"string\",
            \"pci_bus_id\": \"string\"
        }
        Pass an empty array if you want to remove GPUs."
    )]
    pub gpu_json_file: std::path::PathBuf,
}
