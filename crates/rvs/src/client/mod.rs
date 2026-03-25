pub mod convert;

use crate::error::RvsError;
use rpc::forge::{GetRackRequest, Machine, MachinesByIdsRequest, Rack};
use rpc::forge_api_client::ForgeApiClient;
use rpc::forge_tls_client::ApiConfig;

/// NVLink fields extracted from gRPC Machine.
#[derive(Debug)]
#[allow(dead_code)]
pub struct TrayNvlData {
    /// NVL domain this tray belongs to.
    pub domain_uuid: Option<String>,
    /// GPU count reported via NVLink info.
    pub gpu_count: u32,
}

/// InfiniBand fields extracted from gRPC Machine.
#[derive(Debug)]
#[allow(dead_code)]
pub struct TrayIbData {
    /// Fabric IDs observed across IB interfaces (used as partition keys).
    pub fabric_ids: Vec<String>,
    /// Total IB interfaces on this machine.
    pub port_count: u32,
    /// IB interfaces with active LID.
    pub active_port_count: u32,
}

/// Intermediate representation of a gRPC Machine.
#[derive(Debug)]
#[allow(dead_code)]
pub struct TrayData {
    /// Machine ID as string.
    pub id: String,
    /// Machine lifecycle state.
    pub state: String,
    /// NVLink data, if machine has NVLink info.
    pub nvl: Option<TrayNvlData>,
    /// InfiniBand data, if machine has IB status.
    pub ib: Option<TrayIbData>,
}

/// Extract TrayData from gRPC Machine.
impl From<Machine> for TrayData {
    fn from(value: Machine) -> Self {
        let id = value
            .id
            .as_ref()
            .map(|id| id.to_string())
            .unwrap_or_default();

        let nvl = value.nvlink_info.map(|info| TrayNvlData {
            domain_uuid: info.domain_uuid.as_ref().map(|u| u.to_string()),
            gpu_count: info.gpus.len() as u32,
        });

        let ib = value.ib_status.map(|status| {
            let port_count = status.ib_interfaces.len() as u32;
            let active_port_count = status
                .ib_interfaces
                .iter()
                .filter(|iface| matches!(iface.lid, Some(lid) if lid != 0 && lid != 0xffff))
                .count() as u32;
            let fabric_ids = status
                .ib_interfaces
                .iter()
                .filter_map(|iface| iface.fabric_id.clone())
                .collect();
            TrayIbData {
                fabric_ids,
                port_count,
                active_port_count,
            }
        });

        Self {
            id,
            state: value.state,
            nvl,
            ib,
        }
    }
}

/// Intermediate representation of a gRPC Rack.
#[derive(Debug)]
#[allow(dead_code)]
pub struct RackData {
    /// Rack ID as string.
    pub id: String,
    /// Rack lifecycle state.
    pub state: String,
    /// Machine IDs of compute trays in this rack.
    pub compute_tray_ids: Vec<String>,
}

/// Extract RackData from gRPC Rack.
impl From<Rack> for RackData {
    fn from(value: Rack) -> Self {
        Self {
            id: value
                .id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_default(),
            state: value.rack_state,
            compute_tray_ids: value
                .compute_trays
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
        }
    }
}

/// BMMC gRPC client wrapper -- translates gRPC responses into IR types.
pub struct BmmcClient {
    inner: ForgeApiClient,
}

impl BmmcClient {
    /// Construct from API config (hardcoded TLS paths for now).
    pub fn new(api_config: &ApiConfig<'_>) -> Self {
        Self {
            inner: ForgeApiClient::new(api_config),
        }
    }

    /// Fetch all racks from BMMC -> Vec<RackData>.
    pub async fn get_racks(&self) -> Result<Vec<RackData>, RvsError> {
        let response = self.inner.get_rack(GetRackRequest { id: None }).await?;
        Ok(response.rack.into_iter().map(RackData::from).collect())
    }

    /// Fetch machines for a rack's compute trays -> Vec<TrayData>. Chunked at 50.
    pub async fn get_machines(&self, rack: &RackData) -> Result<Vec<TrayData>, RvsError> {
        let mut trays = Vec::with_capacity(rack.compute_tray_ids.len());

        for chunk in rack.compute_tray_ids.chunks(50) {
            let machine_ids = chunk
                .iter()
                .map(|id| {
                    id.parse()
                        .map_err(|_| RvsError::InvalidMachineId(id.clone()))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let response = self
                .inner
                .find_machines_by_ids(MachinesByIdsRequest {
                    machine_ids,
                    include_history: false,
                })
                .await?;

            trays.extend(response.machines.into_iter().map(TrayData::from));
        }

        Ok(trays)
    }
}