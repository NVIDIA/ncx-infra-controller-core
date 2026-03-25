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

use std::collections::HashMap;
use std::sync::Arc;

use carbide_uuid::machine::MachineId;
use config_version::Versioned;
use eyre::eyre;
use itertools::Itertools;
use model::instance::InstanceNetworkSyncStatus;
use model::instance::config::network::{
    DeviceLocator, InstanceInterfaceConfig, InterfaceFunctionId, NetworkDetails,
};
use model::instance::snapshot::InstanceSnapshot;
use model::instance::status::SyncState;
use model::instance::status::extension_service::InstanceExtensionServicesStatus;
use model::machine::{
    InstanceState, ManagedHostState, ManagedHostStateSnapshot, NetworkConfigUpdateState,
};
use model::resource_pool::common::CommonPools;
use sqlx::PgConnection;

use super::super::check_host_health_for_alerts;
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

/// Checks if an instance's network is synced and its DPU is healthy.
///
/// This function compares the expected network configuration version with the actual version.
/// It also checks the health of the DPU by calling `check_host_health_for_alerts`.
///
/// # Notes
/// This function currently does not support multi-DPU handling.
pub(super) fn check_instance_network_synced_and_dpu_healthy(
    instance: &InstanceSnapshot,
    mh_snapshot: &ManagedHostStateSnapshot,
) -> Result<InstanceNetworkSyncStatus, StateHandlerError> {
    if mh_snapshot
        .host_snapshot
        .associated_dpu_machine_ids()
        .is_empty()
    {
        tracing::info!(
            machine_id = %mh_snapshot.host_snapshot.id,
            "Skipping network config because machine has no DPUs"
        );
        return Ok(InstanceNetworkSyncStatus::ZeroDpuNoObservationNeeded);
    }

    let device_locators: Vec<DeviceLocator> = instance
        .config
        .network
        .interfaces
        .iter()
        .filter_map(|i| i.device_locator.clone())
        .collect();

    let maps = mh_snapshot
        .host_snapshot
        .get_dpu_device_and_id_mappings()
        .unwrap_or_else(|_| (HashMap::default(), HashMap::default()));

    let legacy_physical_interface_count = instance
        .config
        .network
        .interfaces
        .iter()
        .filter(|iface| {
            iface.function_id == InterfaceFunctionId::Physical {} && iface.device_locator.is_none()
        })
        .count();

    let use_primary_dpu_only = legacy_physical_interface_count > 0
        || device_locators.is_empty()
        || maps.0.is_empty()
        || maps.1.is_empty();

    let dpu_machine_ids: Vec<MachineId> = if use_primary_dpu_only {
        if legacy_physical_interface_count != 1 {
            return Err(StateHandlerError::GenericError(eyre!(
                "More than one interface configured when only the primary dpu is allowed"
            )));
        }
        // allow primary dpu to be used when using one config with no device_locators
        match mh_snapshot
            .host_snapshot
            .interfaces
            .iter()
            .find(|iface| iface.primary_interface)
            .and_then(|iface| iface.attached_dpu_machine_id)
        {
            Some(primary_dpu_id) => vec![primary_dpu_id],
            None => {
                return Err(StateHandlerError::GenericError(eyre!(
                    "Could not find primary dpu id"
                )));
            }
        }
    } else {
        if maps.0.is_empty() || maps.1.is_empty() {
            return Err(StateHandlerError::GenericError(eyre!(
                "No interface device locators for when using multiple interfaces"
            )));
        }

        let id_to_device_map = maps.0;
        let device_to_id_map = maps.1;
        // filter out dpus that do not have interfaces configured
        mh_snapshot
            .host_snapshot
            .associated_dpu_machine_ids()
            .iter()
            .filter(|dpu_machine_id| {
                if let Some(device) = id_to_device_map.get(dpu_machine_id) {
                    tracing::info!("Found device {} for dpu {}", device, dpu_machine_id);
                    if let Some(id_vec) = device_to_id_map.get(device)
                        && let Some(device_instance) =
                            id_vec.iter().position(|id| id == *dpu_machine_id)
                    {
                        tracing::info!(
                            "Found device_instance {} for dpu {}",
                            device_instance,
                            dpu_machine_id
                        );
                        let device_locator = DeviceLocator {
                            device: device.clone(),
                            device_instance,
                        };
                        return instance.config.network.interfaces.iter().any(|i| {
                            i.device_locator
                                .as_ref()
                                .is_some_and(|dl| dl == &device_locator)
                        });
                    }
                }
                false
            })
            .copied()
            .collect()
    };

    if instance.observations.network.len() != dpu_machine_ids.len() {
        tracing::info!(
            "obs: {} dpus: {}",
            instance.observations.network.len(),
            dpu_machine_ids.len()
        );

        let mut missing_dpus = Vec::default();
        for dpu_id in dpu_machine_ids {
            tracing::info!("checking dpu: {}", dpu_id);
            if !instance.observations.network.contains_key(&dpu_id) {
                tracing::info!("missing");
                missing_dpus.push(dpu_id);
            }
        }
        return Ok(InstanceNetworkSyncStatus::InstanceNetworkObservationNotAvailable(missing_dpus));
    }
    // Check instance network config has been applied
    let expected = &instance.network_config_version;

    let mut outdated_dpus = Vec::default();
    for (dpu_machine_id, network_obs) in &instance.observations.network {
        if &network_obs.config_version != expected {
            outdated_dpus.push(*dpu_machine_id);
        }
    }

    if !outdated_dpus.is_empty() {
        return Ok(InstanceNetworkSyncStatus::InstanceNetworkNotSynced(
            outdated_dpus,
        ));
    }

    check_host_health_for_alerts(mh_snapshot)?;
    Ok(InstanceNetworkSyncStatus::InstanceNetworkSynced)
}

pub async fn release_vpc_dpu_loopback(
    mh_snapshot: &ManagedHostStateSnapshot,
    common_pools: Option<&CommonPools>,
    txn: &mut PgConnection,
) -> Result<(), StateHandlerError> {
    for dpu_snapshot in &mh_snapshot.dpu_snapshots {
        if let Some(common_pools) = common_pools {
            db::vpc_dpu_loopback::delete_and_deallocate(common_pools, &dpu_snapshot.id, txn, false)
                .await
                .map_err(|e| StateHandlerError::ResourceCleanupError {
                    resource: "VpcLoopbackIp",
                    error: e.to_string(),
                })?;
        }
    }

    Ok(())
}

pub(super) async fn release_network_segments_with_vpc_prefix(
    interfaces: &[InstanceInterfaceConfig],
    txn: &mut PgConnection,
) -> Result<(), StateHandlerError> {
    let network_segment_ids_with_vpc = interfaces
        .iter()
        .filter_map(|x| match x.network_details {
            Some(NetworkDetails::VpcPrefixId(_)) => x.network_segment_id,
            _ => None,
        })
        .collect_vec();

    // Mark all network ready for delete which were created for vpc_prefixes.
    if !network_segment_ids_with_vpc.is_empty() {
        db::network_segment::mark_as_deleted_no_validation(txn, &network_segment_ids_with_vpc)
            .await
            .map_err(|err| StateHandlerError::ResourceCleanupError {
                resource: "network_segment",
                error: err.to_string(),
            })?;
    }

    Ok(())
}

// Gets extension services status from DB, checks if any removed services are fully terminated
// across all DPUs, if so, remove them from the instance config in the DB(without updating the version).
pub(super) fn get_extension_services_status(
    mh_snapshot: &ManagedHostStateSnapshot,
    instance: &InstanceSnapshot,
) -> InstanceExtensionServicesStatus {
    let (_, dpu_id_to_device_map) = mh_snapshot
        .host_snapshot
        .get_dpu_device_and_id_mappings()
        .unwrap_or_else(|_| (HashMap::default(), HashMap::default()));

    // Gather instance extension services status from all DPU observations
    InstanceExtensionServicesStatus::from_config_and_observations(
        &dpu_id_to_device_map,
        Versioned::new(
            &instance.config.extension_services,
            instance.extension_services_config_version,
        ),
        &instance.observations.extension_services,
    )
}

pub(super) async fn cleanup_terminated_extension_services(
    instance: &InstanceSnapshot,
    extension_services_status: &mut InstanceExtensionServicesStatus,
    txn: &mut PgConnection,
) -> Result<(), StateHandlerError> {
    if extension_services_status.configs_synced != SyncState::Synced {
        return Ok(());
    }

    let terminated_service_ids = extension_services_status.get_terminated_service_ids();
    if terminated_service_ids.is_empty() {
        return Ok(());
    }

    tracing::info!(
        instance_id = %instance.id,
        service_ids = ?terminated_service_ids,
        "Cleaning up fully terminated extension services from instance config"
    );
    let new_config = instance
        .config
        .extension_services
        .remove_terminated_services(&terminated_service_ids);

    db::instance::update_extension_services_config(
        txn,
        instance.id,
        instance.extension_services_config_version,
        &new_config,
        false,
    )
    .await?;

    extension_services_status
        .extension_services
        .retain(|svc| !terminated_service_ids.contains(&svc.service_id));
    Ok(())
}

pub(super) async fn handle_instance_network_config_update_request(
    mh_snapshot: &ManagedHostStateSnapshot,
    network_config_update_state: &NetworkConfigUpdateState,
    instance: &InstanceSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
    common_pools: &Option<Arc<CommonPools>>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    match network_config_update_state {
        NetworkConfigUpdateState::WaitingForNetworkSegmentToBeReady => {
            let next_state = ManagedHostState::Assigned {
                instance_state: InstanceState::NetworkConfigUpdate {
                    network_config_update_state: NetworkConfigUpdateState::WaitingForConfigSynced,
                },
            };

            let Some(update_request) = &instance.update_network_config_request else {
                return Err(StateHandlerError::GenericError(eyre::eyre!(
                    "Network config update request is missing from db. instance: {}",
                    instance.id
                )));
            };

            let network_segment_ids_with_vpc = update_request
                .new_config
                .interfaces
                .iter()
                .filter_map(|x| match x.network_details {
                    Some(NetworkDetails::VpcPrefixId(_)) => x.network_segment_id,
                    _ => None,
                })
                .collect_vec();

            // No network segment is configured with vpc_prefix_id.
            if !network_segment_ids_with_vpc.is_empty() {
                let network_segments_are_ready = db::network_segment::are_network_segments_ready(
                    &mut ctx.services.db_reader,
                    &network_segment_ids_with_vpc,
                )
                .await?;
                if !network_segments_are_ready {
                    return Ok(StateHandlerOutcome::wait(
                        "Waiting for all segments to come in ready state.".to_string(),
                    ));
                }
            }

            // Update requested network config and increment version.
            let mut txn = ctx.services.db_pool.begin().await?;
            db::instance::update_network_config(
                txn.as_mut(),
                instance.id,
                instance.network_config_version,
                &update_request.new_config,
                true,
            )
            .await?;

            Ok(StateHandlerOutcome::transition(next_state).with_txn(txn))
        }
        NetworkConfigUpdateState::WaitingForConfigSynced => {
            let next_state = ManagedHostState::Assigned {
                instance_state: InstanceState::NetworkConfigUpdate {
                    network_config_update_state: NetworkConfigUpdateState::ReleaseOldResources,
                },
            };

            Ok(
                match check_instance_network_synced_and_dpu_healthy(instance, mh_snapshot)? {
                    InstanceNetworkSyncStatus::InstanceNetworkObservationNotAvailable(
                        missing_dpus,
                    ) => StateHandlerOutcome::wait(format!(
                        "Waiting for DPU agents to apply initial network config for DPUs: {}",
                        missing_dpus.iter().map(|dpu| dpu.to_string()).join(", ")
                    )),
                    InstanceNetworkSyncStatus::ZeroDpuNoObservationNeeded
                    | InstanceNetworkSyncStatus::InstanceNetworkSynced => {
                        StateHandlerOutcome::transition(next_state)
                    }
                    InstanceNetworkSyncStatus::InstanceNetworkNotSynced(outdated_dpus) => {
                        StateHandlerOutcome::wait(format!(
                            "Waiting for DPU agent to apply most recent network config for DPUs: {}",
                            outdated_dpus.iter().map(|dpu| dpu.to_string()).join(", ")
                        ))
                    }
                },
            )
        }
        NetworkConfigUpdateState::ReleaseOldResources => {
            let mut txn = ctx.services.db_pool.begin().await?;
            // Identify all the resources which have to be released.
            // Release Ips.
            // Release segments.
            // Release VpcDpuLoopbackIps.
            // Free the update_network_config_request field.
            let Some(update_request) = &instance.update_network_config_request else {
                return Err(StateHandlerError::GenericError(eyre::eyre!(
                    "Network config update request is missing from db. instance: {}",
                    instance.id
                )));
            };

            // Logically new_config is current_config now.
            let mut new_config = update_request.new_config.clone();
            let copied_resources = new_config.copy_existing_resources(&update_request.old_config);

            let resources_to_be_released = update_request
                .old_config
                .interfaces
                .iter()
                .filter(|x| !copied_resources.contains(x))
                .cloned()
                .collect_vec();

            if !resources_to_be_released.is_empty() {
                let addresses = resources_to_be_released
                    .iter()
                    .flat_map(|x| x.ip_addrs.values().copied().collect_vec())
                    .collect_vec();

                tracing::info!(
                    "Releasing network resources for instance {}: addresses: {:?}",
                    instance.id,
                    addresses,
                );
                db::instance_address::delete_addresses(&mut txn, &addresses).await?;
                release_network_segments_with_vpc_prefix(&resources_to_be_released, &mut txn)
                    .await?;

                // TODO: This is not the best way, but will work fine. If you delete all loopback IPs
                // associated with all DPUs, dpu_agent will assign new IPs during next managed_host_network_config
                // iteration.
                // The best way would be to find out the VPCs per DPU which are not used in new config
                // and delete them only. This can be taken care once multi-dpu instance allocation is
                // completed.
                release_vpc_dpu_loopback(mh_snapshot, common_pools.as_deref(), &mut txn).await?;
            }
            db::instance::delete_update_network_config_request(&instance.id, &mut txn).await?;
            let next_state = ManagedHostState::Assigned {
                instance_state: InstanceState::Ready,
            };
            Ok(StateHandlerOutcome::transition(next_state).with_txn(txn))
        }
    }
}
