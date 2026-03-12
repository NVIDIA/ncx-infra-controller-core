use std::collections::HashMap;

use crate::error::RvsError;
use crate::partitions::{IbTray, NvlTray};
use crate::rack::{Rack, Racks, Tray};

use super::TrayData;

/// Bundled result of fetching one rack's data from BMMC.
#[derive(Debug)]
pub struct FetchedRack {
    /// Rack ID this data belongs to.
    pub rack_id: String,
    /// Resolved tray data for this rack's compute trays.
    pub trays: Vec<TrayData>,
}

impl FetchedRack {
    /// Construct from rack ID and its fetched tray data.
    pub fn new(rack_id: String, trays: Vec<TrayData>) -> Self {
        Self { rack_id, trays }
    }
}

/// Build a resolved Rack from IR tray data -- groups trays by NVL domain and IB fabric.
impl TryFrom<Vec<TrayData>> for Rack {
    type Error = RvsError;

    fn try_from(tray_data: Vec<TrayData>) -> Result<Self, Self::Error> {
        let mut nvl: HashMap<String, Vec<String>> = HashMap::new();
        let mut ib: HashMap<String, Vec<String>> = HashMap::new();
        let mut all: HashMap<String, Tray> = HashMap::new();

        for tray in tray_data {
            if let Some(ref nvl_data) = tray.nvl {
                if let Some(domain_uuid) = &nvl_data.domain_uuid {
                    nvl.entry(domain_uuid.clone())
                        .or_default()
                        .push(tray.id.clone());
                }
            }

            if let Some(ref ib_data) = tray.ib {
                for fabric_id in &ib_data.fabric_ids {
                    ib.entry(fabric_id.clone())
                        .or_default()
                        .push(tray.id.clone());
                }
            }

            let gpu_count = tray.nvl.as_ref().map(|n| n.gpu_count).unwrap_or(0);
            let (ib_port_count, ib_active_port_count) = tray
                .ib
                .as_ref()
                .map(|i| (i.port_count, i.active_port_count))
                .unwrap_or((0, 0));

            let model_tray = Tray::new(
                NvlTray::new(gpu_count),
                IbTray::new(ib_port_count, ib_active_port_count),
            );
            all.insert(tray.id, model_tray);
        }

        Ok(Rack::new(nvl, ib, all))
    }
}

/// Build resolved Racks collection from fetched rack bundles.
impl TryFrom<Vec<FetchedRack>> for Racks {
    type Error = RvsError;

    fn try_from(value: Vec<FetchedRack>) -> Result<Self, Self::Error> {
        let mut inner: HashMap<String, Rack> = HashMap::new();

        for fetched in value {
            let rack = Rack::try_from(fetched.trays)?;
            inner.insert(fetched.rack_id, rack);
        }

        Ok(Racks::new(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use super::super::{TrayIbData, TrayNvlData};

    /// Test helper -- build a TrayData with optional NVL/IB fields.
    fn tray(id: &str, domain: Option<&str>, gpus: u32, ib_fabrics: &[&str]) -> TrayData {
        let nvl = if domain.is_some() || gpus > 0 {
            Some(TrayNvlData {
                domain_uuid: domain.map(|d| d.to_string()),
                gpu_count: gpus,
            })
        } else {
            None
        };

        let ib = if !ib_fabrics.is_empty() {
            Some(TrayIbData {
                fabric_ids: ib_fabrics.iter().map(|f| f.to_string()).collect(),
                port_count: ib_fabrics.len() as u32,
                active_port_count: ib_fabrics.len() as u32,
            })
        } else {
            None
        };

        TrayData {
            id: id.to_string(),
            state: "ready".to_string(),
            nvl,
            ib,
        }
    }

    #[test]
    fn test_trays_to_rack() {
        let trays = vec![
            tray("m1", Some("domain-a"), 4, &["fabric-1"]),
            tray("m2", Some("domain-a"), 4, &["fabric-1"]),
            tray("m3", Some("domain-b"), 4, &["fabric-1", "fabric-2"]),
        ];

        let rack = Rack::try_from(trays).unwrap();

        assert_eq!(rack.all().len(), 3);
        // NVLink grouping
        assert_eq!(rack.nvl().len(), 2);
        assert_eq!(rack.nvl()["domain-a"], vec!["m1", "m2"]);
        assert_eq!(rack.nvl()["domain-b"], vec!["m3"]);
        // IB grouping -- m3 appears in both fabrics
        assert_eq!(rack.ib().len(), 2);
        assert_eq!(rack.ib()["fabric-1"], vec!["m1", "m2", "m3"]);
        assert_eq!(rack.ib()["fabric-2"], vec!["m3"]);
    }

    #[test]
    fn test_fetched_racks_to_racks() {
        let fetched = vec![
            FetchedRack::new("rack-1".to_string(), vec![tray("m1", Some("d1"), 4, &["f1"])]),
            FetchedRack::new("rack-2".to_string(), vec![tray("m2", Some("d2"), 8, &["f2"])]),
        ];

        let racks = Racks::try_from(fetched).unwrap();

        assert_eq!(racks.inner().len(), 2);
        assert!(racks.inner().contains_key("rack-1"));
        assert!(racks.inner().contains_key("rack-2"));
        assert_eq!(racks.inner()["rack-1"].all().len(), 1);
        assert_eq!(racks.inner()["rack-2"].all().len(), 1);
    }
}
