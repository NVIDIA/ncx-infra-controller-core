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

use std::str::FromStr;

use carbide_uuid::machine::MachineId;
use chrono::Utc;
use eyre::eyre;
use health_report::{
    HealthAlertClassification, HealthProbeAlert, HealthProbeId, HealthReport, OverrideMode,
};
use itertools::Itertools;
use libredfish::SystemPowerControl;
use model::instance::InstanceNetworkSyncStatus;
use model::instance::config::network::NetworkDetails;
use model::instance::status::SyncState;
use model::instance::status::extension_service::{
    self, ExtensionServiceDeploymentStatus, ExtensionServicesReadiness,
};
use model::machine::infiniband::{IbConfigNotSyncedReason, ib_config_synced};
use model::machine::nvlink::nvlink_config_synced;
use model::machine::{
    CleanupState, HostPlatformConfigurationState, HostReprovisionState, InstanceNextStateResolver,
    InstanceState, ManagedHostState, ManagedHostStateSnapshot, MeasuringState,
    NetworkConfigUpdateState, NextStateBFBSupport, ReprovisionState, RetryInfo, UnlockHostState,
};

use super::super::helpers::ReprovisionStateHelper;
use super::super::host_reprovision_state::HostFirmwareScenario;
use super::super::{
    are_dpus_up_trigger_reboot_if_needed, check_host_health_for_alerts, dpu_reprovisioning_needed,
    handler_host_power_control, handler_restart_dpu, host_power_state, rebooted,
    set_managed_host_topology_update_needed, trigger_reboot_if_needed,
};
use super::InstanceStateHandler;
use super::helpers::{
    check_instance_network_synced_and_dpu_healthy, cleanup_terminated_extension_services,
    get_extension_services_status, handle_instance_network_config_update_request,
    release_network_segments_with_vpc_prefix, release_vpc_dpu_loopback,
};
use crate::state_controller::machine::context::MachineStateHandlerContextObjects;
use crate::state_controller::machine::write_ops::MachineWriteOp;
use crate::state_controller::state_handler::{
    StateHandlerContext, StateHandlerError, StateHandlerOutcome,
};

pub(super) async fn handle(
    handler: &InstanceStateHandler,
    host_machine_id: &MachineId,
    mh_snapshot: &mut ManagedHostStateSnapshot,
    ctx: &mut StateHandlerContext<'_, MachineStateHandlerContextObjects>,
) -> Result<StateHandlerOutcome<ManagedHostState>, StateHandlerError> {
    let Some(ref instance) = mh_snapshot.instance else {
        return Err(StateHandlerError::GenericError(eyre!(
            "Instance is empty at this point. Cleanup is needed for host: {}.",
            host_machine_id
        )));
    };

    if let ManagedHostState::Assigned { instance_state } = &mh_snapshot.managed_state {
        match instance_state {
            InstanceState::Init => {
                // we should not be here. This state to be used if state machine has not
                // picked instance creation and user asked for status.
                Err(StateHandlerError::InvalidHostState(
                    *host_machine_id,
                    Box::new(mh_snapshot.managed_state.clone()),
                ))
            }
            InstanceState::WaitingForNetworkSegmentToBeReady => {
                let next_state = ManagedHostState::Assigned {
                    instance_state: InstanceState::WaitingForNetworkConfig,
                };
                let network_segment_ids_with_vpc = instance
                    .config
                    .network
                    .interfaces
                    .iter()
                    .filter_map(|x| match x.network_details {
                        Some(NetworkDetails::VpcPrefixId(_)) => x.network_segment_id,
                        _ => None,
                    })
                    .collect_vec();

                // No network segment is configured with vpc_prefix_id.
                if network_segment_ids_with_vpc.is_empty() {
                    return Ok(StateHandlerOutcome::transition(next_state));
                }

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
                Ok(StateHandlerOutcome::transition(next_state))
            }
            InstanceState::WaitingForNetworkConfig => {
                // It should be first state to process here.
                // Wait for instance network config to be applied
                // Reboot host and moved to Ready.

                // TODO GK if delete_requested skip this whole step,
                // reboot and jump to BootingWithDiscoveryImage

                // Check DPU network config has been applied
                if !mh_snapshot.managed_host_network_config_version_synced() {
                    return Ok(StateHandlerOutcome::wait(
                                "Waiting for DPU agent(s) to apply network config and report healthy network"
                                    .to_string()
                            ));
                }

                let next_state = ManagedHostState::Assigned {
                    instance_state: InstanceState::WaitingForRebootToReady,
                };

                // Check instance network config has been applied
                match check_instance_network_synced_and_dpu_healthy(instance, mh_snapshot)? {
                    InstanceNetworkSyncStatus::InstanceNetworkObservationNotAvailable(
                        missing_dpus,
                    ) => {
                        return Ok(StateHandlerOutcome::wait(format!(
                            "Waiting for DPU agents to apply initial network config for DPUs: {}",
                            missing_dpus.iter().map(|dpu| dpu.to_string()).join(", ")
                        )));
                    }
                    InstanceNetworkSyncStatus::InstanceNetworkSynced => {}
                    InstanceNetworkSyncStatus::ZeroDpuNoObservationNeeded => {
                        return Ok(StateHandlerOutcome::transition(next_state));
                    }
                    InstanceNetworkSyncStatus::InstanceNetworkNotSynced(outdated_dpus) => {
                        return Ok(StateHandlerOutcome::wait(format!(
                            "Waiting for DPU agent to apply most recent network config for DPUs: {}",
                            outdated_dpus.iter().map(|dpu| dpu.to_string()).join(", ")
                        )));
                    }
                };

                // Check whether the IB config is synced
                if let Err(not_synced_reason) = ib_config_synced(
                    mh_snapshot
                        .host_snapshot
                        .infiniband_status_observation
                        .as_ref(),
                    Some(&instance.config.infiniband),
                    true,
                ) {
                    return Ok(StateHandlerOutcome::wait(format!(
                        "Waiting for IB config to be applied: {}",
                        not_synced_reason
                    )));
                }

                // Check if the nvlink config has been applied
                if let Err(not_synced_reason) = nvlink_config_synced(
                    mh_snapshot.host_snapshot.nvlink_status_observation.as_ref(),
                    Some(&instance.config.nvlink),
                ) {
                    return Ok(StateHandlerOutcome::wait(format!(
                        "Waiting for NvLink config to be applied: {}",
                        not_synced_reason.0
                    )));
                }
                Ok(StateHandlerOutcome::transition(next_state))
            }
            InstanceState::WaitingForStorageConfig => {
                // This state used to do something but doesn't any more, we can delete
                // InstanceState::WaitingForStorageConfig once we're sure no places have the
                // state persisted.
                let next_state = ManagedHostState::Assigned {
                    instance_state: InstanceState::WaitingForExtensionServicesConfig,
                };
                Ok(StateHandlerOutcome::transition(next_state))
            }
            InstanceState::WaitingForExtensionServicesConfig => {
                // If no extension services are configured, skip the wait and proceed
                if instance
                    .config
                    .extension_services
                    .service_configs
                    .is_empty()
                {
                    let next_state = ManagedHostState::Assigned {
                        instance_state: InstanceState::WaitingForRebootToReady,
                    };
                    return Ok(StateHandlerOutcome::transition(next_state));
                }

                let mut extension_services_status =
                    get_extension_services_status(mh_snapshot, instance);
                let txn = if extension_services_status.configs_synced == SyncState::Synced
                    && !extension_services_status
                        .get_terminated_service_ids()
                        .is_empty()
                {
                    let mut txn = ctx.services.db_pool.begin().await?;
                    cleanup_terminated_extension_services(
                        instance,
                        &mut extension_services_status,
                        txn.as_mut(),
                    )
                    .await?;

                    Some(txn)
                } else {
                    None
                };
                let outcome = match extension_service::compute_extension_services_readiness(&extension_services_status) {
                            ExtensionServicesReadiness::Ready => {
                                let next_state = ManagedHostState::Assigned {
                                    instance_state: InstanceState::WaitingForRebootToReady,
                                };
                                StateHandlerOutcome::transition(next_state)
                            }
                            ExtensionServicesReadiness::ConfigsPending => {
                                StateHandlerOutcome::wait(
                                    "Waiting for extension services config to be applied on all DPUs.".to_string(),
                                )
                            }
                            ExtensionServicesReadiness::NotFullyRunning => {
                                StateHandlerOutcome::wait(
                                    "Waiting for all active extension services to be running on all DPUs.".to_string(),
                                )
                            }
                            ExtensionServicesReadiness::SomeTerminating => {
                                StateHandlerOutcome::wait(
                                    "Waiting for all terminating extension services to be fully terminated across all DPUs."
                                        .to_string(),
                                )
                            }
                        };
                Ok(match txn {
                    Some(txn) => outcome.with_txn(txn),
                    None => outcome,
                })
            }
            InstanceState::WaitingForRebootToReady => {
                // If custom_pxe_reboot_requested is set, this reboot was triggered by
                // the tenant requested a boot with custom iPXE. Clear the request flag.
                // The use_custom_pxe_on_boot flag was already set by the API handler.
                if instance.custom_pxe_reboot_requested {
                    ctx.pending_db_writes
                        .push(MachineWriteOp::SetCustomPxeRebootRequested {
                            machine_id: mh_snapshot.host_snapshot.id,
                            requested: false,
                        });
                }

                // Reboot host
                handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart)
                    .await?;

                // Instance is ready.
                // We can not determine if machine is rebooted successfully or not. Just leave
                // it like this and declare Instance Ready.
                let next_state = ManagedHostState::Assigned {
                    instance_state: InstanceState::Ready,
                };
                Ok(StateHandlerOutcome::transition(next_state))
            }
            InstanceState::Ready => {
                // Machine is up after reboot. Hurray. Instance is up.

                // Wait for user's approval. Once user approves for dpu
                // reprovision/update firmware, trigger it.
                let is_auto_approved = handler.host_upgrade.is_auto_approved();

                // We will give first priority to network config update.
                // This is the easiest way to stop resource leakage.
                if instance.update_network_config_request.is_some() {
                    // Tenant has requested network config update.
                    let next_state = ManagedHostState::Assigned {
                        instance_state: InstanceState::NetworkConfigUpdate {
                            network_config_update_state:
                                NetworkConfigUpdateState::WaitingForNetworkSegmentToBeReady,
                        },
                    };
                    return Ok(StateHandlerOutcome::transition(next_state));
                }

                // Run cleanup here so fully terminated extension services are
                // removed from persisted instance config.
                let mut txn_opt = None;
                if !instance
                    .config
                    .extension_services
                    .service_configs
                    .is_empty()
                {
                    let mut extension_services_status =
                        get_extension_services_status(mh_snapshot, instance);
                    if extension_services_status.configs_synced == SyncState::Synced
                        && !extension_services_status
                            .get_terminated_service_ids()
                            .is_empty()
                    {
                        let mut txn = ctx.services.db_pool.begin().await?;
                        cleanup_terminated_extension_services(
                            instance,
                            &mut extension_services_status,
                            txn.as_mut(),
                        )
                        .await?;
                        txn_opt = Some(txn);
                    }
                }

                let reprov_can_be_started = if dpu_reprovisioning_needed(&mh_snapshot.dpu_snapshots)
                {
                    // Usually all DPUs are updated with user_approval_received field as true
                    // if `invoke_instance_power` is called.
                    // TODO: multidpu: Move this field to `instances` table and unset on
                    // reprovision is completed.
                    mh_snapshot
                        .dpu_snapshots
                        .iter()
                        .filter(|x| x.reprovision_requested.is_some())
                        .all(|x| {
                            x.reprovision_requested
                                .as_ref()
                                .map(|x| x.user_approval_received || is_auto_approved)
                                .unwrap_or_default()
                        })
                } else {
                    false
                };
                let host_firmware_requested =
                    if let Some(request) = &mh_snapshot.host_snapshot.host_reprovision_requested {
                        request.user_approval_received || is_auto_approved
                    } else {
                        false
                    };

                if is_auto_approved && (reprov_can_be_started || host_firmware_requested) {
                    tracing::info!(machine_id = %host_machine_id, "Auto rebooting host for reprovision/upgrade due to being in approved time period");
                }

                // Check if the instance needs to PXE boot. The custom_pxe_reboot_requested flag
                // is set by the API when the tenant calls InvokeInstancePower with boot_with_custom_ipxe=true
                //
                // This triggers the HostPlatformConfiguration flow to verify BIOS boot order
                // before rebooting. The WaitingForRebootToReady handler will clear this flag
                // and set use_custom_pxe_on_boot, which the iPXE handler uses to serve the
                // tenant's script.
                let boot_with_custom_ipxe = instance.custom_pxe_reboot_requested;

                if instance.deleted.is_some()
                    || reprov_can_be_started
                    || host_firmware_requested
                    || boot_with_custom_ipxe
                {
                    for dpu_snapshot in &mh_snapshot.dpu_snapshots {
                        if dpu_snapshot.reprovision_requested.is_some() {
                            // User won't be allowed to clear reprovisioning flag after this.
                            ctx.pending_db_writes.push(
                                MachineWriteOp::UpdateDpuReprovisionStartTime {
                                    machine_id: dpu_snapshot.id,
                                    time: Utc::now(),
                                },
                            );
                        }
                    }
                    if mh_snapshot
                        .host_snapshot
                        .host_reprovision_requested
                        .is_some()
                    {
                        ctx.pending_db_writes.push(
                            MachineWriteOp::UpdateHostReprovisionStartTime {
                                machine_id: mh_snapshot.host_snapshot.id,
                                time: Utc::now(),
                            },
                        );
                    }

                    // For deletion, power cycle the host first. For everything else
                    // (reprovision, firmware update, custom PXE), verify boot config first.
                    let next_state = if instance.deleted.is_some() {
                        let redfish_client = ctx
                            .services
                            .create_redfish_client_from_machine(&mh_snapshot.host_snapshot)
                            .await?;

                        let power_state = host_power_state(redfish_client.as_ref()).await?;

                        ManagedHostState::Assigned {
                            instance_state: InstanceState::HostPlatformConfiguration {
                                platform_config_state: HostPlatformConfigurationState::PowerCycle {
                                    power_on: power_state == libredfish::PowerState::Off,
                                    power_on_retry_count: 0,
                                },
                            },
                        }
                    } else {
                        ManagedHostState::Assigned {
                            instance_state: InstanceState::HostPlatformConfiguration {
                                platform_config_state: HostPlatformConfigurationState::UnlockHost {
                                    unlock_host_state: UnlockHostState::DisableLockdown,
                                },
                            },
                        }
                    };

                    let mut txn = if let Some(txn) = txn_opt.take() {
                        txn
                    } else {
                        ctx.services.db_pool.begin().await?
                    };

                    if host_firmware_requested {
                        let health_override =
                                    crate::machine_update_manager::machine_update_module::create_host_update_health_report_hostfw();
                        let machine_id = *host_machine_id;
                        // The health report alert gets generated here, the machine update manager retains responsibilty for clearing it when we're done.
                        db::machine::insert_health_report_override(
                            &mut txn,
                            &machine_id,
                            OverrideMode::Merge,
                            &health_override,
                            false,
                        )
                        .await?;
                    }

                    if reprov_can_be_started {
                        let health_override = crate::machine_update_manager::machine_update_module::create_host_update_health_report_dpufw();
                        let machine_id = *host_machine_id;
                        // Mark the Host as in update.
                        db::machine::insert_health_report_override(
                            &mut txn,
                            &machine_id,
                            OverrideMode::Merge,
                            &health_override,
                            false,
                        )
                        .await?;
                    }

                    Ok(StateHandlerOutcome::transition(next_state).with_txn(txn))
                } else if let Some(txn) = txn_opt {
                    Ok(StateHandlerOutcome::do_nothing().with_txn(txn))
                } else {
                    Ok(StateHandlerOutcome::do_nothing())
                }
            }
            InstanceState::HostPlatformConfiguration {
                platform_config_state,
            } => {
                super::super::host_machine_state::handle_instance_host_platform_config(
                    ctx,
                    mh_snapshot,
                    &handler.reachability_params,
                    platform_config_state.clone(),
                )
                .await
            }
            InstanceState::WaitingForDpusToUp => {
                if !are_dpus_up_trigger_reboot_if_needed(
                    mh_snapshot,
                    &handler.reachability_params,
                    ctx,
                )
                .await
                {
                    return Ok(StateHandlerOutcome::wait(
                        "Waiting for DPUs to come up.".to_string(),
                    ));
                }

                // If custom_pxe_reboot_requested is set, transition to WaitingForRebootToReady and reboot.
                // The iPXE handler will then serve the tenant's custom script when the host PXE boots.
                //
                // The API sets custom_pxe_reboot_requested when the tenant explicitly requests
                // "Reboot with Custom iPXE"
                //
                // Otherwise, follow the normal termination/reprovision flow through
                // BootingWithDiscoveryImage.
                if instance.custom_pxe_reboot_requested {
                    if !instance
                        .config
                        .os
                        .run_provisioning_instructions_on_every_boot
                    {
                        ctx.pending_db_writes
                            .push(MachineWriteOp::UseCustomIpxeOnNextBoot {
                                machine_id: mh_snapshot.host_snapshot.id,
                                boot_with_custom_ipxe: true,
                            });
                    }

                    let next_state = ManagedHostState::Assigned {
                        instance_state: InstanceState::WaitingForRebootToReady,
                    };
                    Ok(StateHandlerOutcome::transition(next_state))
                } else {
                    handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart)
                        .await?;
                    let next_state = ManagedHostState::Assigned {
                        instance_state: InstanceState::BootingWithDiscoveryImage {
                            retry: RetryInfo { count: 0 },
                        },
                    };
                    Ok(StateHandlerOutcome::transition(next_state))
                }
            }
            InstanceState::BootingWithDiscoveryImage { retry } => {
                if !rebooted(&mh_snapshot.host_snapshot) {
                    let status = trigger_reboot_if_needed(
                        &mh_snapshot.host_snapshot,
                        mh_snapshot,
                        // can't send 0. 0 will force power-off as cycle calculator.
                        Some(retry.count as i64 + 1),
                        &handler.reachability_params,
                        ctx,
                    )
                    .await?;

                    let st = if status.increase_retry_count {
                        let next_state = ManagedHostState::Assigned {
                            instance_state: InstanceState::BootingWithDiscoveryImage {
                                retry: RetryInfo {
                                    count: retry.count + 1,
                                },
                            },
                        };
                        StateHandlerOutcome::transition(next_state)
                    } else {
                        StateHandlerOutcome::wait(status.status)
                    };
                    return Ok(st);
                }

                // Now retry_count won't exceed a limit. Function trigger_reboot_if_needed does
                // not reboot a machine after 6 hrs, so this counter won't increase at all
                // after 6 hours.
                ctx.metrics
                    .machine_reboot_attempts_in_booting_with_discovery_image =
                    Some(retry.count + 1);

                // In case state is triggered for delete instance handling, follow that path.
                if instance.deleted.is_some() {
                    let next_state = ManagedHostState::Assigned {
                        instance_state: InstanceState::SwitchToAdminNetwork,
                    };
                    return Ok(StateHandlerOutcome::transition(next_state));
                }

                // If we are here, DPU reprov MUST have been be requested.
                if dpu_reprovisioning_needed(&mh_snapshot.dpu_snapshots) {
                    // All DPUs must have same value for this parameter. All DPUs are updated
                    // together grpc API or automatic updater.
                    // TODO: multidpu: Keep it at some common place to avoid duplicates.
                    let mut dpus_for_reprov = vec![];
                    for dpu_snapshot in &mh_snapshot.dpu_snapshots {
                        if dpu_snapshot.reprovision_requested.is_some() {
                            handler_restart_dpu(
                                dpu_snapshot,
                                ctx,
                                mh_snapshot.host_snapshot.dpf.used_for_ingestion,
                            )
                            .await?;
                            dpus_for_reprov.push(dpu_snapshot);
                        }
                    }

                    set_managed_host_topology_update_needed(
                        ctx.pending_db_writes,
                        &mh_snapshot.host_snapshot,
                        &dpus_for_reprov,
                    );

                    let next_state = ReprovisionState::next_substate_based_on_bfb_support(
                        handler.enable_secure_boot,
                        mh_snapshot,
                        ctx.services.site_config.dpf.enabled,
                    )
                    .next_state_with_all_dpus_updated(
                        &mh_snapshot.managed_state,
                        &mh_snapshot.dpu_snapshots,
                        dpus_for_reprov.iter().map(|x| &x.id).collect_vec(),
                    )?;
                    Ok(StateHandlerOutcome::transition(next_state))
                } else if mh_snapshot
                    .host_snapshot
                    .host_reprovision_requested
                    .is_some()
                {
                    Ok(StateHandlerOutcome::transition(
                        ManagedHostState::Assigned {
                            instance_state: InstanceState::HostReprovision {
                                reprovision_state: HostReprovisionState::CheckingFirmwareV2 {
                                    firmware_type: None,
                                    firmware_number: None,
                                },
                            },
                        },
                    ))
                } else {
                    Ok(StateHandlerOutcome::wait(
                        "Don't know how did we reach here.".to_string(),
                    ))
                }
            }

            InstanceState::SwitchToAdminNetwork => {
                // Tenant is gone and so is their network, switch back to admin network
                let mut txn = ctx.services.db_pool.begin().await?;
                for dpu_snapshot in &mh_snapshot.dpu_snapshots {
                    let (mut netconf, version) = dpu_snapshot.network_config.clone().take();
                    netconf.use_admin_network = Some(true);
                    db::machine::try_update_network_config(
                        &mut txn,
                        &dpu_snapshot.id,
                        version,
                        &netconf,
                    )
                    .await?;
                }

                // Machine is currently an instance, but the instance is being released and we
                // are switching the NICs to the admin network. Set use_admin_network to true
                // and update the network config version in the DPA interfaces. This will cause
                // the DPA State Controller to send SetVNI commands with the VNI being zero.
                for dpa_interface in &mh_snapshot.dpa_interface_snapshots {
                    let (mut netconf, version) = dpa_interface.network_config.clone().take();
                    netconf.use_admin_network = Some(true);
                    let dpa_interface_id = dpa_interface.id;
                    db::dpa_interface::try_update_network_config(
                        &mut txn,
                        &dpa_interface_id,
                        version,
                        &netconf,
                    )
                    .await?;
                }

                let next_state = ManagedHostState::Assigned {
                    instance_state: InstanceState::WaitingForNetworkReconfig,
                };
                Ok(StateHandlerOutcome::transition(next_state).with_txn(txn))
            }
            InstanceState::WaitingForNetworkReconfig => {
                // Has forge-dpu-agent applied the new network config so that
                // we are back on the admin network?
                if !mh_snapshot.managed_host_network_config_version_synced() {
                    return Ok(StateHandlerOutcome::wait(
                                "Waiting for DPU agent(s) to apply network config and report healthy network"
                                    .to_string()
                            ));
                }

                // Check if all DPUs have terminated all extension services
                if let Some(instance) = mh_snapshot.instance.as_ref()
                    && !instance
                        .config
                        .extension_services
                        .service_configs
                        .is_empty()
                {
                    for (_dpu_id, extension_service_statuses) in
                        instance.observations.extension_services.iter()
                    {
                        for status in extension_service_statuses.extension_service_statuses.iter() {
                            if status.overall_state != ExtensionServiceDeploymentStatus::Terminated
                            {
                                return Ok(StateHandlerOutcome::wait(
                                    "Waiting for extension services to be terminated on all DPUs."
                                        .to_string(),
                                ));
                            }
                        }
                    }
                }

                // Check each DPA interface associated with the machine to make sure the DPA NIC has updated
                // its network config (setting VNI to zero in this case).
                if ctx.services.site_config.is_dpa_enabled() {
                    for dpa_interface in &mh_snapshot.dpa_interface_snapshots {
                        if !dpa_interface.managed_host_network_config_version_synced() {
                            return Ok(StateHandlerOutcome::wait(
                                        "Waiting for DPA agent(s) to apply network config and report healthy network"
                                            .to_string()
                                    ));
                        }
                    }
                }

                check_host_health_for_alerts(mh_snapshot)?;

                // Check whether IB config is removed
                match ib_config_synced(
                    mh_snapshot
                        .host_snapshot
                        .infiniband_status_observation
                        .as_ref(),
                    Some(&instance.config.infiniband),
                    false,
                ) {
                    Ok(()) => {
                        // Config is synced, proceed with termination
                    }
                    Err(IbConfigNotSyncedReason::PortStateUnobservable { guids, details }) => {
                        tracing::warn!(
                            instance_id = %instance.id,
                            machine_id = %host_machine_id,
                            guids = ?guids,
                            details = %details,
                            "IB ports not observable during termination - IB Monitor will unbind"
                        );

                        // Collect GUIDs for cleanup
                        // TODO: Include fabric name for multi-fabric deployments
                        let message = format!(
                            "IB port cleanup pending - IB Monitor will unbind. GUIDs: {}",
                            guids.join("; ")
                        );

                        // Create health report with alert that will prevent re-allocation
                        // IB Monitor will unbind before clearing
                        let health_report = HealthReport {
                            source: "ib-cleanup-validation".to_string(),
                            triggered_by: None,
                            observed_at: Some(chrono::Utc::now()),
                            alerts: vec![HealthProbeAlert {
                                id: HealthProbeId::from_str("IbCleanupPending")
                                    .expect("valid probe id"),
                                target: None,
                                in_alert_since: Some(chrono::Utc::now()),
                                message,
                                tenant_message: None,
                                classifications: vec![
                                    HealthAlertClassification::prevent_allocations(),
                                ],
                            }],
                            successes: vec![],
                        };

                        // Use health report override instead of state_controller_health_report field
                        // This is ok to defer into pending_db_writes because we're passing
                        // `no_overwrite: false`, meaning we will overwrite any overrides
                        // already in place.
                        ctx.pending_db_writes
                            .push(MachineWriteOp::InsertHealthReportOverride {
                                machine_id: *host_machine_id,
                                mode: health_report::OverrideMode::Merge,
                                health_report,
                            });

                        tracing::info!(
                            machine_id = %host_machine_id,
                            guids = ?guids,
                            "IbCleanupPending alert created - IB Monitor will handle unbind and clear alert"
                        );

                        // Termination proceeds - IB Monitor will handle cleanup
                    }
                    Err(other_reason) => {
                        return Ok(StateHandlerOutcome::wait(format!(
                            "Waiting for IB config to be removed (Reason: {})",
                            other_reason
                        )));
                    }
                }

                // TODO: TPM cleanup
                // Reboot host
                handler_host_power_control(mh_snapshot, ctx, SystemPowerControl::ForceRestart)
                    .await?;

                // Deleting an instance and marking vpc segments deleted must be done together.
                // If segments are marked deleted and instance is not deleted (may be due to redfish failure),
                // network segment handler will delete those segments forcefully.
                // if instance is deleted before, we won't get network segment details as these
                // details are stored in instance's network config which is deleted.

                // Delete from database now. Once done, reboot and move to next state.
                let mut txn = ctx.services.db_pool.begin().await?;
                db::instance::delete(instance.id, &mut txn)
                    .await
                    .map_err(|err| StateHandlerError::GenericError(err.into()))?;

                release_network_segments_with_vpc_prefix(
                    &instance.config.network.interfaces,
                    &mut txn,
                )
                .await?;

                // Free up all loopback IPs allocated for this instance.
                release_vpc_dpu_loopback(mh_snapshot, handler.common_pools.as_deref(), &mut txn)
                    .await?;

                let next_state = if handler.attestation_enabled {
                    ManagedHostState::PostAssignedMeasuring {
                        measuring_state: MeasuringState::WaitingForMeasurements,
                    }
                } else {
                    ManagedHostState::WaitingForCleanup {
                        cleanup_state: CleanupState::Init,
                    }
                };

                Ok(StateHandlerOutcome::transition(next_state).with_txn(txn))
            }
            InstanceState::DPUReprovision { .. } => {
                for dpu_snapshot in &mh_snapshot.dpu_snapshots {
                    if let outcome @ StateHandlerOutcome::Transition { .. } =
                        super::super::managed_host_state::dpu_reprovision::handle_dpu_reprovision(
                            mh_snapshot,
                            &handler.reachability_params,
                            &InstanceNextStateResolver,
                            dpu_snapshot,
                            ctx,
                            &handler.hardware_models,
                            handler.dpf_sdk.as_deref(),
                        )
                        .await?
                    {
                        return Ok(outcome);
                    }
                }
                Ok(StateHandlerOutcome::do_nothing())
            }
            InstanceState::Failed {
                details,
                machine_id,
            } => {
                // Only way to proceed is to
                // 1. Force-delete the machine.
                // 2. If failed during reprovision, fix the config/hw issue and
                //    retrigger DPU reprovision.
                tracing::warn!(
                    "Instance id {}/machine: {} stuck in failed state. details: {:?}, failed machine: {}",
                    instance.id,
                    host_machine_id,
                    details,
                    machine_id
                );
                Ok(StateHandlerOutcome::do_nothing())
            }
            InstanceState::HostReprovision { .. } => {
                handler
                    .host_upgrade
                    .handle_host_reprovision(
                        mh_snapshot,
                        ctx,
                        host_machine_id,
                        HostFirmwareScenario::Instance,
                    )
                    .await
            }
            InstanceState::NetworkConfigUpdate {
                network_config_update_state,
            } => {
                handle_instance_network_config_update_request(
                    mh_snapshot,
                    network_config_update_state,
                    instance,
                    ctx,
                    &handler.common_pools,
                )
                .await
            }
            InstanceState::DpaProvisioning => {
                // An instance is being created.
                // So we set use_admin_network to false and tell each DPA interface to
                // update its network config. This will cause the DPA state controller
                // to transition to the DPAs from READY state to WaitingForSetVNI state
                // and send SetVNI commands to the DPA NICs.

                let mut txn = ctx.services.db_pool.begin().await?;
                if ctx.services.site_config.is_dpa_enabled() {
                    for dpa_interface in &mh_snapshot.dpa_interface_snapshots {
                        let (mut netconf, version) = dpa_interface.network_config.clone().take();
                        netconf.use_admin_network = Some(false);
                        db::dpa_interface::try_update_network_config(
                            &mut txn,
                            &dpa_interface.id,
                            version,
                            &netconf,
                        )
                        .await?;
                    }
                }
                let next_state = ManagedHostState::Assigned {
                    instance_state: InstanceState::WaitingForDpaToBeReady,
                };
                Ok(StateHandlerOutcome::transition(next_state).with_txn(txn))
            }
            InstanceState::WaitingForDpaToBeReady => {
                // Check each DPA interface to see if it has acted on updating the network config.
                // This involves the DPA State Machine sending SetVNI commands to the NICs, and getting
                // an ACK. If any of the interfaces has not yet heard back the ACk, we will continue to
                // be in the current state.
                if ctx.services.site_config.is_dpa_enabled() {
                    for dpa_interface in &mh_snapshot.dpa_interface_snapshots {
                        if !dpa_interface.managed_host_network_config_version_synced() {
                            return Ok(StateHandlerOutcome::wait(
                                        "Waiting for DPA agent(s) to apply network config and report healthy network"
                                            .to_string()
                                    ));
                        }
                    }
                }

                // Switch to using the network we just created for the tenant
                let mut txn = ctx.services.db_pool.begin().await?;
                for dpu_snapshot in &mh_snapshot.dpu_snapshots {
                    let (mut netconf, version) = dpu_snapshot.network_config.clone().take();
                    netconf.use_admin_network = Some(false);
                    db::machine::try_update_network_config(
                        &mut txn,
                        &dpu_snapshot.id,
                        version,
                        &netconf,
                    )
                    .await?;
                }

                let next_state = ManagedHostState::Assigned {
                    instance_state: InstanceState::WaitingForNetworkSegmentToBeReady,
                };
                Ok(StateHandlerOutcome::transition(next_state).with_txn(txn))
            }
        }
    } else {
        // We are not in Assigned state. Should this be Err(StateHandlerError::InvalidHostState)?
        Ok(StateHandlerOutcome::do_nothing())
    }
}
